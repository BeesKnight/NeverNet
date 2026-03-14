use axum::{http::Request, middleware::Next, response::Response};
use tokio::task_local;

task_local! {
    static REQUEST_ID: String;
}

pub async fn with_request_context(request: Request<axum::body::Body>, next: Next) -> Response {
    let request_id = request
        .headers()
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("missing")
        .to_string();

    REQUEST_ID.scope(request_id, next.run(request)).await
}

pub fn current_request_id() -> Option<String> {
    REQUEST_ID.try_with(Clone::clone).ok()
}

#[cfg(test)]
mod tests {
    use axum::{
        Router,
        body::{Body, to_bytes},
        http::Request,
        middleware,
        routing::get,
    };
    use tower::util::ServiceExt;

    use super::*;

    #[test]
    fn request_id_is_absent_outside_context() {
        assert_eq!(current_request_id(), None);
    }

    #[tokio::test]
    async fn middleware_scopes_request_id_for_handler() {
        let app = Router::new()
            .route(
                "/",
                get(|| async { current_request_id().unwrap_or_default() }),
            )
            .layer(middleware::from_fn(with_request_context));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/")
                    .header("x-request-id", "req-123")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();

        assert_eq!(String::from_utf8(body.to_vec()).unwrap(), "req-123");
    }
}
