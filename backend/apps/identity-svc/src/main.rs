mod app_state;
mod auth;
mod config;
mod error;
mod users;

use std::sync::Arc;

use app_state::AppState;
use auth::models::{LoginRequest, RegisterRequest};
use contracts::identity::identity_service_server::{IdentityService, IdentityServiceServer};
use contracts::identity::{AuthReply, CurrentUserRequest, Empty, LogoutRequest, User, UserReply};
use persistence::connect_pool;
use tonic::{Request, Response, Status, transport::Server};

use crate::{config::Config, error::AppError, users::models::UserProfile};

#[derive(Clone)]
struct IdentityGrpcService {
    state: AppState,
}

#[tonic::async_trait]
impl IdentityService for IdentityGrpcService {
    async fn register(
        &self,
        request: Request<contracts::identity::RegisterRequest>,
    ) -> Result<Response<AuthReply>, Status> {
        let span = observability::grpc_request_span("identity.register", &request);
        tracing::info!(parent: &span, "grpc request received");
        let payload = RegisterRequest {
            email: request.get_ref().email.clone(),
            password: request.get_ref().password.clone(),
            full_name: request.get_ref().full_name.clone(),
        };
        let auth = auth::service::register(&self.state, payload)
            .await
            .map_err(status_from_error)?;

        Ok(Response::new(AuthReply {
            token: auth.token,
            user: Some(map_user(auth.user)),
        }))
    }

    async fn login(
        &self,
        request: Request<contracts::identity::LoginRequest>,
    ) -> Result<Response<AuthReply>, Status> {
        let span = observability::grpc_request_span("identity.login", &request);
        tracing::info!(parent: &span, "grpc request received");
        let payload = LoginRequest {
            email: request.get_ref().email.clone(),
            password: request.get_ref().password.clone(),
        };
        let auth = auth::service::login(&self.state, payload)
            .await
            .map_err(status_from_error)?;

        Ok(Response::new(AuthReply {
            token: auth.token,
            user: Some(map_user(auth.user)),
        }))
    }

    async fn logout(&self, request: Request<LogoutRequest>) -> Result<Response<Empty>, Status> {
        let span = observability::grpc_request_span("identity.logout", &request);
        tracing::info!(parent: &span, "grpc request received");
        auth::service::logout(&self.state, &request.get_ref().token)
            .await
            .map_err(status_from_error)?;
        Ok(Response::new(Empty {}))
    }

    async fn get_current_user(
        &self,
        request: Request<CurrentUserRequest>,
    ) -> Result<Response<UserReply>, Status> {
        let span = observability::grpc_request_span("identity.current_user", &request);
        tracing::info!(parent: &span, "grpc request received");
        let user = auth::service::get_current_user(&self.state, &request.get_ref().token)
            .await
            .map_err(status_from_error)?;

        Ok(Response::new(UserReply {
            user: Some(map_user(user)),
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    observability::init_tracing("identity-svc", "identity_svc=info");

    let config = Arc::new(Config::from_env()?);
    observability::spawn_metrics_server("identity-svc", config.metrics_port);
    let pool = connect_pool(&config.database_url, 5).await?;
    let state = AppState::new(pool, config.clone());
    let address = format!("0.0.0.0:{}", config.grpc_port).parse()?;

    tracing::info!("identity-svc listening on {address}");

    Server::builder()
        .add_service(IdentityServiceServer::new(IdentityGrpcService { state }))
        .serve(address)
        .await?;

    Ok(())
}

fn map_user(user: UserProfile) -> User {
    User {
        id: user.id.to_string(),
        email: user.email,
        full_name: user.full_name,
        created_at: user.created_at.to_rfc3339(),
    }
}

fn status_from_error(error: AppError) -> Status {
    match error {
        AppError::BadRequest(message) => Status::invalid_argument(message),
        AppError::Unauthorized(message) => Status::unauthenticated(message),
        AppError::NotFound(message) => Status::not_found(message),
        AppError::Conflict(message) => Status::already_exists(message),
        AppError::Config(message) | AppError::Internal(message) => Status::internal(message),
        AppError::Database(inner) => {
            tracing::error!("identity database error: {}", inner);
            Status::internal("Database operation failed")
        }
        AppError::Migration(inner) => {
            tracing::error!("identity migration error: {}", inner);
            Status::internal("Migration failed")
        }
    }
}
