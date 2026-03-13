use tonic::{
    metadata::MetadataValue,
    service::Interceptor,
    transport::{Channel, Endpoint},
};

use crate::{error::AppError, shared::request_context};

#[derive(Clone, Default)]
pub struct RequestIdInterceptor;

impl Interceptor for RequestIdInterceptor {
    fn call(
        &mut self,
        mut request: tonic::Request<()>,
    ) -> Result<tonic::Request<()>, tonic::Status> {
        if let Some(request_id) = request_context::current_request_id() {
            let value = MetadataValue::try_from(request_id.as_str())
                .map_err(|_| tonic::Status::internal("Could not serialize request id metadata"))?;
            request.metadata_mut().insert("x-request-id", value);
        }

        Ok(request)
    }
}

pub async fn connect_channel(endpoint: &str, service_name: &str) -> Result<Channel, AppError> {
    Endpoint::from_shared(endpoint.to_string())
        .map_err(|error| AppError::Internal(format!("{service_name} address is invalid: {error}")))?
        .connect()
        .await
        .map_err(|error| AppError::Internal(format!("{service_name} is unavailable: {error}")))
}
