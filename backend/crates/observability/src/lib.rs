use std::{env, net::SocketAddr, sync::LazyLock, time::Duration};

use axum::{Router, routing::get};
use http::StatusCode;
use prometheus::{
    Encoder, GaugeVec, HistogramOpts, HistogramVec, IntCounterVec, Opts, Registry, TextEncoder,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

static REGISTRY: LazyLock<Registry> = LazyLock::new(Registry::new);
static SERVICE_INFO: LazyLock<GaugeVec> = LazyLock::new(|| {
    register(
        GaugeVec::new(
            Opts::new(
                "eventdesign_service_info",
                "Static metadata about the running EventDesign service",
            ),
            &["service", "version"],
        )
        .expect("service info metric"),
    )
});
static HTTP_REQUESTS_TOTAL: LazyLock<IntCounterVec> = LazyLock::new(|| {
    register(
        IntCounterVec::new(
            Opts::new(
                "eventdesign_http_requests_total",
                "Count of HTTP requests handled by the service",
            ),
            &["method", "route", "status"],
        )
        .expect("http request counter"),
    )
});
static HTTP_REQUEST_DURATION: LazyLock<HistogramVec> = LazyLock::new(|| {
    register(
        HistogramVec::new(
            HistogramOpts::new(
                "eventdesign_http_request_duration_seconds",
                "End-to-end HTTP request latency",
            ),
            &["method", "route", "status"],
        )
        .expect("http request duration histogram"),
    )
});
static CACHE_REQUESTS_TOTAL: LazyLock<IntCounterVec> = LazyLock::new(|| {
    register(
        IntCounterVec::new(
            Opts::new(
                "eventdesign_cache_requests_total",
                "Count of cache hits and misses",
            ),
            &["cache", "result"],
        )
        .expect("cache request counter"),
    )
});
static SECURITY_EVENTS_TOTAL: LazyLock<IntCounterVec> = LazyLock::new(|| {
    register(
        IntCounterVec::new(
            Opts::new(
                "eventdesign_security_events_total",
                "Count of rejected or suspicious security-related events",
            ),
            &["kind"],
        )
        .expect("security event counter"),
    )
});
static EXPORT_DURATION: LazyLock<HistogramVec> = LazyLock::new(|| {
    register(
        HistogramVec::new(
            HistogramOpts::new(
                "eventdesign_export_duration_seconds",
                "Duration of export job execution",
            ),
            &["format", "status"],
        )
        .expect("export duration histogram"),
    )
});
static PROJECTION_LAG: LazyLock<GaugeVec> = LazyLock::new(|| {
    register(
        GaugeVec::new(
            Opts::new(
                "eventdesign_projection_lag_seconds",
                "Observed lag between write-side events and projection processing",
            ),
            &["projection"],
        )
        .expect("projection lag gauge"),
    )
});
static QUEUE_LAG: LazyLock<GaugeVec> = LazyLock::new(|| {
    register(
        GaugeVec::new(
            Opts::new(
                "eventdesign_worker_queue_lag_seconds",
                "Observed lag between queue publication and worker processing",
            ),
            &["queue"],
        )
        .expect("queue lag gauge"),
    )
});

pub fn init_tracing(service_name: &str, default_filter: &str) {
    SERVICE_INFO
        .with_label_values(&[service_name, env!("CARGO_PKG_VERSION")])
        .set(1.0);

    let filter = tracing_subscriber::EnvFilter::new(
        env::var("RUST_LOG").unwrap_or_else(|_| default_filter.to_string()),
    );
    let log_format = env::var("LOG_FORMAT")
        .unwrap_or_else(|_| "pretty".to_string())
        .to_lowercase();

    if log_format == "json" {
        tracing_subscriber::registry()
            .with(filter)
            .with(
                tracing_subscriber::fmt::layer()
                    .json()
                    .flatten_event(true)
                    .with_current_span(true)
                    .with_span_list(true),
            )
            .init();
    } else {
        tracing_subscriber::registry()
            .with(filter)
            .with(
                tracing_subscriber::fmt::layer()
                    .compact()
                    .with_target(true)
                    .with_thread_ids(true),
            )
            .init();
    }
}

pub fn spawn_metrics_server(service_name: &'static str, port: u16) {
    let app = Router::new()
        .route(
            "/metrics",
            get(|| async move {
                let metric_families = REGISTRY.gather();
                let encoder = TextEncoder::new();
                let mut buffer = Vec::new();

                match encoder.encode(&metric_families, &mut buffer) {
                    Ok(()) => (
                        StatusCode::OK,
                        [("content-type", encoder.format_type().to_string())],
                        String::from_utf8(buffer).unwrap_or_default(),
                    ),
                    Err(error) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        [("content-type", "text/plain; charset=utf-8".to_string())],
                        format!("could not encode metrics: {error}"),
                    ),
                }
            }),
        )
        .route("/healthz", get(|| async { "ok" }));

    tokio::spawn(async move {
        let address = SocketAddr::from(([0, 0, 0, 0], port));

        match tokio::net::TcpListener::bind(address).await {
            Ok(listener) => {
                tracing::info!("{service_name} metrics listening on http://{address}");
                if let Err(error) = axum::serve(listener, app).await {
                    tracing::error!("{service_name} metrics server failed: {error}");
                }
            }
            Err(error) => {
                tracing::error!("could not bind metrics listener for {service_name}: {error}");
            }
        }
    });
}

pub fn observe_http_request(method: &str, route: &str, status: u16, duration: Duration) {
    let status = status.to_string();
    HTTP_REQUESTS_TOTAL
        .with_label_values(&[method, route, &status])
        .inc();
    HTTP_REQUEST_DURATION
        .with_label_values(&[method, route, &status])
        .observe(duration.as_secs_f64());
}

pub fn observe_cache_result(cache: &str, result: &str) {
    CACHE_REQUESTS_TOTAL
        .with_label_values(&[cache, result])
        .inc();
}

pub fn increment_security_event(kind: &str) {
    SECURITY_EVENTS_TOTAL.with_label_values(&[kind]).inc();
}

pub fn observe_export_duration(format: &str, status: &str, duration: Duration) {
    EXPORT_DURATION
        .with_label_values(&[format, status])
        .observe(duration.as_secs_f64());
}

pub fn set_projection_lag(projection: &str, seconds: f64) {
    PROJECTION_LAG
        .with_label_values(&[projection])
        .set(seconds.max(0.0));
}

pub fn set_queue_lag(queue: &str, seconds: f64) {
    QUEUE_LAG.with_label_values(&[queue]).set(seconds.max(0.0));
}

pub fn grpc_request_span<T>(method: &'static str, request: &tonic::Request<T>) -> tracing::Span {
    let request_id = request
        .metadata()
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("missing");

    tracing::info_span!("grpc_request", method, request_id)
}

fn register<T>(collector: T) -> T
where
    T: prometheus::core::Collector + Clone + Send + Sync + 'static,
{
    REGISTRY
        .register(Box::new(collector.clone()))
        .expect("collector registration should succeed exactly once");
    collector
}
