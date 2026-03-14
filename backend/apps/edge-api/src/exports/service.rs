use axum::{
    body::Body,
    http::{
        StatusCode,
        header::{CONTENT_DISPOSITION, CONTENT_TYPE, HeaderValue},
    },
    response::Response,
};
use contracts::reporting::report_service_client::ReportServiceClient;
use contracts::reporting::{
    CreateExportRequest as ReportCreateExportRequest, DownloadExportRequest, GetExportRequest,
    ListExportsRequest,
};
use tonic::transport::Channel;
use uuid::Uuid;

use crate::{
    app_state::AppState,
    error::AppError,
    exports::models::{CreateExportRequest, ExportJob},
    shared::grpc,
};

pub async fn list(state: &AppState, user_id: Uuid) -> Result<Vec<ExportJob>, AppError> {
    let mut client = report_client(state).await?;
    let reply = client
        .list_exports(ListExportsRequest {
            user_id: user_id.to_string(),
        })
        .await
        .map_err(map_status)?
        .into_inner();

    reply.items.into_iter().map(map_export_job).collect()
}

pub async fn get(state: &AppState, user_id: Uuid, export_id: Uuid) -> Result<ExportJob, AppError> {
    let mut client = report_client(state).await?;
    let reply = client
        .get_export(GetExportRequest {
            user_id: user_id.to_string(),
            export_id: export_id.to_string(),
        })
        .await
        .map_err(map_status)?
        .into_inner();

    map_export_job(
        reply.job.ok_or_else(|| {
            AppError::Internal("Report response is missing export job".to_string())
        })?,
    )
}

pub async fn create(
    state: &AppState,
    user_id: Uuid,
    payload: CreateExportRequest,
) -> Result<ExportJob, AppError> {
    let mut client = report_client(state).await?;
    let filters_json = serde_json::to_string(&payload.filters)
        .map_err(|error| AppError::Internal(error.to_string()))?;
    let reply = client
        .create_export(ReportCreateExportRequest {
            user_id: user_id.to_string(),
            report_type: payload.report_type,
            format: payload.format,
            filters_json,
        })
        .await
        .map_err(map_status)?
        .into_inner();

    map_export_job(
        reply.job.ok_or_else(|| {
            AppError::Internal("Report response is missing export job".to_string())
        })?,
    )
}

pub async fn download(
    state: &AppState,
    user_id: Uuid,
    export_id: Uuid,
) -> Result<Response, AppError> {
    let mut client = report_client(state).await?;
    let reply = client
        .download_export(DownloadExportRequest {
            user_id: user_id.to_string(),
            export_id: export_id.to_string(),
        })
        .await
        .map_err(map_status)?
        .into_inner();

    Response::builder()
        .status(StatusCode::OK)
        .header(
            CONTENT_TYPE,
            HeaderValue::from_str(&reply.content_type)
                .map_err(|error| AppError::Internal(error.to_string()))?,
        )
        .header(
            CONTENT_DISPOSITION,
            HeaderValue::from_str(&format!("attachment; filename=\"{}\"", reply.file_name))
                .map_err(|error| AppError::Internal(error.to_string()))?,
        )
        .body(Body::from(reply.bytes))
        .map_err(AppError::from)
}

async fn report_client(
    state: &AppState,
) -> Result<
    ReportServiceClient<
        tonic::service::interceptor::InterceptedService<Channel, grpc::RequestIdInterceptor>,
    >,
    AppError,
> {
    let channel = grpc::connect_channel(&state.config.report_service_url, "Report service").await?;

    Ok(ReportServiceClient::with_interceptor(
        channel,
        grpc::RequestIdInterceptor,
    ))
}

fn map_export_job(job: contracts::reporting::ExportJob) -> Result<ExportJob, AppError> {
    Ok(ExportJob {
        id: parse_uuid(&job.id, "export id")?,
        user_id: parse_uuid(&job.user_id, "export user id")?,
        report_type: job.report_type,
        format: job.format,
        status: job.status,
        filters: serde_json::from_str(&job.filters_json).map_err(|_| {
            AppError::Internal("Internal service returned invalid export filters".to_string())
        })?,
        object_key: optional_string(job.object_key),
        content_type: optional_string(job.content_type),
        error_message: optional_string(job.error_message),
        created_at: parse_timestamp(&job.created_at, "export created_at")?,
        started_at: optional_timestamp(&job.started_at, "export started_at")?,
        updated_at: parse_timestamp(&job.updated_at, "export updated_at")?,
        finished_at: optional_timestamp(&job.finished_at, "export finished_at")?,
    })
}

fn optional_string(value: String) -> Option<String> {
    if value.is_empty() { None } else { Some(value) }
}

fn parse_uuid(value: &str, field: &str) -> Result<Uuid, AppError> {
    Uuid::parse_str(value)
        .map_err(|_| AppError::Internal(format!("Internal service returned an invalid {field}")))
}

fn parse_timestamp(value: &str, field: &str) -> Result<chrono::DateTime<chrono::Utc>, AppError> {
    chrono::DateTime::parse_from_rfc3339(value)
        .map(|timestamp| timestamp.with_timezone(&chrono::Utc))
        .map_err(|_| AppError::Internal(format!("Internal service returned an invalid {field}")))
}

fn optional_timestamp(
    value: &str,
    field: &str,
) -> Result<Option<chrono::DateTime<chrono::Utc>>, AppError> {
    if value.is_empty() {
        Ok(None)
    } else {
        parse_timestamp(value, field).map(Some)
    }
}

fn map_status(status: tonic::Status) -> AppError {
    match status.code() {
        tonic::Code::InvalidArgument => AppError::BadRequest(status.message().to_string()),
        tonic::Code::NotFound => AppError::NotFound(status.message().to_string()),
        tonic::Code::FailedPrecondition => AppError::BadRequest(status.message().to_string()),
        _ => AppError::Internal(format!("Report service error: {}", status.message())),
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::*;

    fn sample_export_job() -> contracts::reporting::ExportJob {
        contracts::reporting::ExportJob {
            id: Uuid::new_v4().to_string(),
            user_id: Uuid::new_v4().to_string(),
            report_type: "summary".to_string(),
            format: "pdf".to_string(),
            status: "completed".to_string(),
            filters_json: r#"{"status":"planned"}"#.to_string(),
            object_key: "exports/report.pdf".to_string(),
            content_type: "application/pdf".to_string(),
            error_message: String::new(),
            created_at: Utc
                .with_ymd_and_hms(2026, 3, 13, 10, 0, 0)
                .unwrap()
                .to_rfc3339(),
            started_at: String::new(),
            updated_at: Utc
                .with_ymd_and_hms(2026, 3, 13, 10, 5, 0)
                .unwrap()
                .to_rfc3339(),
            finished_at: Utc
                .with_ymd_and_hms(2026, 3, 13, 10, 6, 0)
                .unwrap()
                .to_rfc3339(),
        }
    }

    #[test]
    fn maps_export_jobs_and_optional_values() {
        let job = map_export_job(sample_export_job()).expect("export job should map");

        assert_eq!(job.report_type, "summary");
        assert_eq!(job.content_type.as_deref(), Some("application/pdf"));
        assert_eq!(optional_string(String::new()), None);
        assert!(
            optional_timestamp("", "started_at")
                .expect("empty timestamp")
                .is_none()
        );
    }

    #[test]
    fn rejects_invalid_export_payload() {
        let mut job = sample_export_job();
        job.filters_json = "{".to_string();

        assert!(map_export_job(job).is_err());
    }

    #[test]
    fn maps_export_status_codes() {
        assert!(matches!(
            map_status(tonic::Status::invalid_argument("bad")),
            AppError::BadRequest(_)
        ));
        assert!(matches!(
            map_status(tonic::Status::not_found("missing")),
            AppError::NotFound(_)
        ));
        assert!(matches!(
            map_status(tonic::Status::failed_precondition("not-ready")),
            AppError::BadRequest(_)
        ));
        assert!(matches!(
            map_status(tonic::Status::internal("oops")),
            AppError::Internal(_)
        ));
    }
}
