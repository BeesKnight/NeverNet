use contracts::reporting::report_service_server::{ReportService, ReportServiceServer};
use contracts::reporting::{CreateExportRequest, GetExportRequest, ReportAck, SummaryRequest};
use tonic::{Request, Response, Status, transport::Server};

#[derive(Default)]
struct ReportCompatibilityService;

#[tonic::async_trait]
impl ReportService for ReportCompatibilityService {
    async fn get_summary(
        &self,
        _request: Request<SummaryRequest>,
    ) -> Result<Response<ReportAck>, Status> {
        Err(phase_one_unimplemented())
    }

    async fn create_export(
        &self,
        _request: Request<CreateExportRequest>,
    ) -> Result<Response<ReportAck>, Status> {
        Err(phase_one_unimplemented())
    }

    async fn get_export_status(
        &self,
        _request: Request<GetExportRequest>,
    ) -> Result<Response<ReportAck>, Status> {
        Err(phase_one_unimplemented())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    observability::init_tracing("report_svc=info");

    let port = std::env::var("GRPC_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(50054);
    let address = format!("0.0.0.0:{port}").parse()?;

    tracing::info!("report-svc compatibility server listening on {address}");

    Server::builder()
        .add_service(ReportServiceServer::new(ReportCompatibilityService))
        .serve(address)
        .await?;

    Ok(())
}

fn phase_one_unimplemented() -> Status {
    Status::unimplemented(
        "Phase 1 compatibility mode: edge-api still executes reporting and export flows directly",
    )
}
