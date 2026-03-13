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
