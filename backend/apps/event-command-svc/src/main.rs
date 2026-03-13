use contracts::event_command::event_command_service_server::{
    EventCommandService, EventCommandServiceServer,
};
use contracts::event_command::{
    CommandAck, CreateCategoryRequest, CreateEventRequest, DeleteCategoryRequest,
    DeleteEventRequest, UpdateCategoryRequest, UpdateEventRequest,
};
use tonic::{Request, Response, Status, transport::Server};

#[derive(Default)]
struct EventCommandCompatibilityService;

#[tonic::async_trait]
impl EventCommandService for EventCommandCompatibilityService {
    async fn create_category(
        &self,
        _request: Request<CreateCategoryRequest>,
    ) -> Result<Response<CommandAck>, Status> {
        Err(phase_one_unimplemented())
    }

    async fn update_category(
        &self,
        _request: Request<UpdateCategoryRequest>,
    ) -> Result<Response<CommandAck>, Status> {
        Err(phase_one_unimplemented())
    }

    async fn delete_category(
        &self,
        _request: Request<DeleteCategoryRequest>,
    ) -> Result<Response<CommandAck>, Status> {
        Err(phase_one_unimplemented())
    }

    async fn create_event(
        &self,
        _request: Request<CreateEventRequest>,
    ) -> Result<Response<CommandAck>, Status> {
        Err(phase_one_unimplemented())
    }

    async fn update_event(
        &self,
        _request: Request<UpdateEventRequest>,
    ) -> Result<Response<CommandAck>, Status> {
        Err(phase_one_unimplemented())
    }

    async fn delete_event(
        &self,
        _request: Request<DeleteEventRequest>,
    ) -> Result<Response<CommandAck>, Status> {
        Err(phase_one_unimplemented())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    observability::init_tracing("event_command_svc=info");

    let port = std::env::var("GRPC_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(50052);
    let address = format!("0.0.0.0:{port}").parse()?;

    tracing::info!("event-command-svc compatibility server listening on {address}");

    Server::builder()
        .add_service(EventCommandServiceServer::new(
            EventCommandCompatibilityService,
        ))
        .serve(address)
        .await?;

    Ok(())
}

fn phase_one_unimplemented() -> Status {
    Status::unimplemented(
        "Phase 1 compatibility mode: edge-api still executes command flows directly",
    )
}
