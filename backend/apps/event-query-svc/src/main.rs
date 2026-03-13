use contracts::event_query::event_query_service_server::{
    EventQueryService, EventQueryServiceServer,
};
use contracts::event_query::{
    GetCalendarRequest, GetDashboardRequest, GetEventRequest, ListCategoriesRequest,
    ListEventsRequest, QueryAck,
};
use tonic::{Request, Response, Status, transport::Server};

#[derive(Default)]
struct EventQueryCompatibilityService;

#[tonic::async_trait]
impl EventQueryService for EventQueryCompatibilityService {
    async fn list_categories(
        &self,
        _request: Request<ListCategoriesRequest>,
    ) -> Result<Response<QueryAck>, Status> {
        Err(phase_one_unimplemented())
    }

    async fn list_events(
        &self,
        _request: Request<ListEventsRequest>,
    ) -> Result<Response<QueryAck>, Status> {
        Err(phase_one_unimplemented())
    }

    async fn get_event(
        &self,
        _request: Request<GetEventRequest>,
    ) -> Result<Response<QueryAck>, Status> {
        Err(phase_one_unimplemented())
    }

    async fn get_calendar(
        &self,
        _request: Request<GetCalendarRequest>,
    ) -> Result<Response<QueryAck>, Status> {
        Err(phase_one_unimplemented())
    }

    async fn get_dashboard(
        &self,
        _request: Request<GetDashboardRequest>,
    ) -> Result<Response<QueryAck>, Status> {
        Err(phase_one_unimplemented())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    observability::init_tracing("event_query_svc=info");

    let port = std::env::var("GRPC_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(50053);
    let address = format!("0.0.0.0:{port}").parse()?;

    tracing::info!("event-query-svc compatibility server listening on {address}");

    Server::builder()
        .add_service(EventQueryServiceServer::new(EventQueryCompatibilityService))
        .serve(address)
        .await?;

    Ok(())
}

fn phase_one_unimplemented() -> Status {
    Status::unimplemented("Phase 1 compatibility mode: edge-api still serves read flows directly")
}
