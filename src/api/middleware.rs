//! Shared middleware layers for the API.

use std::time::Instant;

use axum::{
    extract::MatchedPath,
    http::{HeaderName, HeaderValue, Request},
    middleware::Next,
    response::IntoResponse,
};
use tower_http::set_header::SetResponseHeaderLayer;

use crate::error::CONTENT_TYPE_SCHEMA_REGISTRY;

// -- Layers --

/// Returns a layer that sets `Content-Type: application/vnd.schemaregistry.v1+json`
/// on every response.
pub fn content_type_layer() -> SetResponseHeaderLayer<HeaderValue> {
    SetResponseHeaderLayer::overriding(
        HeaderName::from_static("content-type"),
        HeaderValue::from_static(CONTENT_TYPE_SCHEMA_REGISTRY),
    )
}

// -- Request instrumentation --

/// Middleware that records Prometheus metrics for every request:
/// - `http_requests_total` counter with `method`, `path`, `status` labels
/// - `http_request_duration_seconds` histogram with `method`, `path` labels
///
/// The `/metrics` endpoint itself is excluded to avoid a self-referential
/// feedback loop that pollutes counters with scraper traffic.
pub async fn track_metrics(req: Request<axum::body::Body>, next: Next) -> impl IntoResponse {
    let method = req.method().to_string();
    let path = req
        .extensions()
        .get::<MatchedPath>()
        .map_or_else(|| "unmatched".to_owned(), |mp| mp.as_str().to_owned());

    let start = Instant::now();
    let response = next.run(req).await;

    // Skip self-instrumentation for the metrics endpoint.
    if path == "/metrics" {
        return response;
    }

    let elapsed = start.elapsed().as_secs_f64();
    let status = response.status().as_u16().to_string();

    metrics::counter!(
        "http_requests_total",
        "method" => method.clone(),
        "path" => path.clone(),
        "status" => status,
    )
    .increment(1);

    metrics::histogram!(
        "http_request_duration_seconds",
        "method" => method,
        "path" => path,
    )
    .record(elapsed);

    response
}
