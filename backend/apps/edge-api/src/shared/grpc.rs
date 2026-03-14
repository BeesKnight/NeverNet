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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interceptor_keeps_request_without_request_id() {
        let mut interceptor = RequestIdInterceptor;
        let request = interceptor
            .call(tonic::Request::new(()))
            .expect("request should pass through");

        assert!(request.metadata().get("x-request-id").is_none());
    }

    #[tokio::test]
    async fn connect_channel_rejects_invalid_endpoints() {
        let error = connect_channel("http://127.0.0.1:1", "identity")
            .await
            .expect_err("unreachable endpoint should fail");

        assert!(error.to_string().contains("identity is unavailable"));
    }
}
