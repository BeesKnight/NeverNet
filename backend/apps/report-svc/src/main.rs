mod app_state;
mod config;
mod error;
mod models;
mod repository;

use std::sync::Arc;

use app_state::AppState;
use contracts::reporting::report_service_server::{ReportService, ReportServiceServer};
use contracts::reporting::{
    CreateExportReply, CreateExportRequest, DownloadExportReply, DownloadExportRequest,
    ExportJob as GrpcExportJob, GetExportReply, GetExportRequest, ListExportsReply,
    ListExportsRequest,
};
use persistence::connect_pool;
use s3::{Bucket, Region, creds::Credentials};
use tonic::{Request, Response, Status, transport::Server};
use uuid::Uuid;

use crate::{
    config::Config,
    error::AppError,
    models::{ExportEventPayload, ExportJob},
};

#[derive(Clone)]
struct ReportGrpcService {
    state: AppState,
}

#[tonic::async_trait]
impl ReportService for ReportGrpcService {
    async fn create_export(
        &self,
        request: Request<CreateExportRequest>,
    ) -> Result<Response<CreateExportReply>, Status> {
        let span = observability::grpc_request_span("report.create_export", &request);
        tracing::info!(parent: &span, "grpc request received");
        let user_id = parse_uuid(&request.get_ref().user_id, "user_id")?;
        let filters = parse_filters_json(&request.get_ref().filters_json)?;
        let report_type = request.get_ref().report_type.trim().to_lowercase();
        let format = request.get_ref().format.trim().to_lowercase();

        validate_report_type(&report_type)?;
        validate_format(&format)?;

        let mut tx = self
            .state
            .pool
            .begin()
            .await
            .map_err(AppError::from)
            .map_err(status_from_error)?;
        let job = repository::create_export_job(&mut tx, user_id, &report_type, &format, filters)
            .await
            .map_err(AppError::from)
            .map_err(status_from_error)?;
        let payload = serde_json::to_value(snapshot_export_payload(&job))
            .map_err(|error| Status::internal(error.to_string()))?;
        repository::insert_outbox_event(&mut tx, job.id, "export.requested", payload)
            .await
            .map_err(AppError::from)
            .map_err(status_from_error)?;
        tx.commit()
            .await
            .map_err(AppError::from)
            .map_err(status_from_error)?;

        Ok(Response::new(CreateExportReply {
            job: Some(map_export_job(job)),
        }))
    }

    async fn list_exports(
        &self,
        request: Request<ListExportsRequest>,
    ) -> Result<Response<ListExportsReply>, Status> {
        let span = observability::grpc_request_span("report.list_exports", &request);
        tracing::info!(parent: &span, "grpc request received");
        let user_id = parse_uuid(&request.get_ref().user_id, "user_id")?;
        let items = repository::list_export_jobs(&self.state.pool, user_id)
            .await
            .map_err(AppError::from)
            .map_err(status_from_error)?
            .into_iter()
            .map(map_export_job)
            .collect();

        Ok(Response::new(ListExportsReply { items }))
    }

    async fn get_export(
        &self,
        request: Request<GetExportRequest>,
    ) -> Result<Response<GetExportReply>, Status> {
        let span = observability::grpc_request_span("report.get_export", &request);
        tracing::info!(parent: &span, "grpc request received");
        let user_id = parse_uuid(&request.get_ref().user_id, "user_id")?;
        let export_id = parse_uuid(&request.get_ref().export_id, "export_id")?;
        let job = repository::get_export_job(&self.state.pool, user_id, export_id)
            .await
            .map_err(AppError::from)
            .map_err(status_from_error)?
            .ok_or_else(|| Status::not_found("Export job not found"))?;

        Ok(Response::new(GetExportReply {
            job: Some(map_export_job(job)),
        }))
    }

    async fn download_export(
        &self,
        request: Request<DownloadExportRequest>,
    ) -> Result<Response<DownloadExportReply>, Status> {
        let span = observability::grpc_request_span("report.download_export", &request);
        tracing::info!(parent: &span, "grpc request received");
        let user_id = parse_uuid(&request.get_ref().user_id, "user_id")?;
        let export_id = parse_uuid(&request.get_ref().export_id, "export_id")?;
        let job = repository::get_export_job(&self.state.pool, user_id, export_id)
            .await
            .map_err(AppError::from)
            .map_err(status_from_error)?
            .ok_or_else(|| Status::not_found("Export job not found"))?;

        if job.status != "completed" {
            return Err(Status::failed_precondition("Export file is not ready yet"));
        }

        let object_key = job
            .object_key
            .clone()
            .ok_or_else(|| Status::internal("Completed export job is missing an object key"))?;
        let content_type = job
            .content_type
            .clone()
            .unwrap_or_else(|| default_content_type(&job.format).to_string());
        let file_name = format!("eventdesign-report-{}.{}", job.id, job.format);
        let object = self
            .state
            .storage
            .get_object(object_key)
            .await
            .map_err(|error| {
                Status::internal(format!("Could not read export artifact: {error}"))
            })?;

        Ok(Response::new(DownloadExportReply {
            bytes: object.into_bytes().to_vec(),
            content_type,
            file_name,
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    observability::init_tracing("report-svc", "report_svc=info");

    let config = Arc::new(Config::from_env()?);
    observability::spawn_metrics_server("report-svc", config.metrics_port);
    let pool = connect_pool(&config.database_url, 10).await?;
    let storage = build_storage_client(&config)?;
    ensure_bucket(&storage, &config.minio_bucket).await;
    let state = AppState::new(pool, storage, config.clone());
    let address = format!("0.0.0.0:{}", config.grpc_port).parse()?;

    tracing::info!("report-svc listening on {address}");

    Server::builder()
        .add_service(ReportServiceServer::new(ReportGrpcService { state }))
        .serve(address)
        .await?;

    Ok(())
}

fn build_storage_client(config: &Config) -> Result<Box<Bucket>, AppError> {
    let region = Region::Custom {
        region: config.minio_region.clone(),
        endpoint: config.minio_endpoint.clone(),
    };
    let credentials = Credentials::new(
        Some(&config.minio_access_key),
        Some(&config.minio_secret_key),
        None,
        None,
        None,
    )
    .map_err(|error| AppError::Internal(format!("Could not build MinIO credentials: {error}")))?;
    Bucket::new(&config.minio_bucket, region, credentials)
        .map(|bucket| bucket.with_path_style())
        .map_err(|error| {
            AppError::Internal(format!("Could not build MinIO bucket client: {error}"))
        })
}

async fn ensure_bucket(client: &Bucket, bucket: &str) {
    if client.exists().await.unwrap_or(false) {
        return;
    }

    unsafe {
        std::env::set_var("RUST_S3_SKIP_LOCATION_CONSTRAINT", "1");
    }
    let credentials = match client.credentials().await {
        Ok(credentials) => credentials,
        Err(error) => {
            tracing::warn!("could not read bucket credentials for {bucket}: {error}");
            return;
        }
    };
    let region = client.region().clone();
    if let Err(error) = Bucket::create_with_path_style(
        bucket,
        region,
        credentials,
        s3::bucket_ops::BucketConfiguration::default(),
    )
    .await
    {
        tracing::warn!("could not ensure bucket {bucket}: {error}");
    }
}

fn snapshot_export_payload(job: &ExportJob) -> ExportEventPayload {
    ExportEventPayload {
        export_id: job.id,
        user_id: job.user_id,
        report_type: job.report_type.clone(),
        format: job.format.clone(),
        status: job.status.clone(),
        filters: job.filters.clone(),
        object_key: job.object_key.clone(),
        error_message: job.error_message.clone(),
        created_at: job.created_at,
        started_at: job.started_at,
        finished_at: job.finished_at,
    }
}

fn map_export_job(job: ExportJob) -> GrpcExportJob {
    GrpcExportJob {
        id: job.id.to_string(),
        user_id: job.user_id.to_string(),
        report_type: job.report_type,
        format: job.format,
        status: job.status,
        filters_json: job.filters.to_string(),
        object_key: job.object_key.unwrap_or_default(),
        content_type: job.content_type.unwrap_or_default(),
        error_message: job.error_message.unwrap_or_default(),
        created_at: job.created_at.to_rfc3339(),
        started_at: job
            .started_at
            .map(|value| value.to_rfc3339())
            .unwrap_or_default(),
        updated_at: job.updated_at.to_rfc3339(),
        finished_at: job
            .finished_at
            .map(|value| value.to_rfc3339())
            .unwrap_or_default(),
    }
}

fn parse_uuid(value: &str, field: &str) -> Result<Uuid, Status> {
    Uuid::parse_str(value).map_err(|_| Status::invalid_argument(format!("Invalid {field}")))
}

fn parse_filters_json(value: &str) -> Result<serde_json::Value, Status> {
    serde_json::from_str(value).map_err(|_| Status::invalid_argument("Invalid filters_json"))
}

fn validate_report_type(value: &str) -> Result<(), Status> {
    if value == "summary" {
        Ok(())
    } else {
        Err(Status::invalid_argument("Report type must be summary"))
    }
}

fn validate_format(value: &str) -> Result<(), Status> {
    if matches!(value, "pdf" | "xlsx") {
        Ok(())
    } else {
        Err(Status::invalid_argument(
            "Export format must be either pdf or xlsx",
        ))
    }
}

fn default_content_type(format: &str) -> &'static str {
    match format {
        "pdf" => "application/pdf",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        _ => "application/octet-stream",
    }
}

fn status_from_error(error: AppError) -> Status {
    match error {
        AppError::Config(message) | AppError::Internal(message) => Status::internal(message),
        AppError::Database(inner) => {
            tracing::error!("report-svc database error: {}", inner);
            Status::internal("Database operation failed")
        }
    }
}
