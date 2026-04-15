//! Integration tests for the Prometheus metrics endpoint.

mod common;

use reqwest::StatusCode;

// -- Endpoint basics --

#[tokio::test]
async fn metrics_returns_200_with_text_content_type() {
    let base = common::spawn_server().await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{base}/metrics"))
        .send()
        .await
        .expect("request should succeed");

    assert_eq!(resp.status(), StatusCode::OK);

    let ct = resp
        .headers()
        .get("content-type")
        .expect("should have content-type")
        .to_str()
        .expect("content-type should be valid utf8");
    assert!(
        ct.starts_with("text/plain"),
        "/metrics should return text/plain, got: {ct}"
    );
}

// -- HTTP instrumentation metrics --

#[tokio::test]
async fn metrics_body_contains_http_request_counters() {
    let base = common::spawn_server().await;
    let client = reqwest::Client::new();

    // Make a request so the counter has data.
    client
        .get(format!("{base}/health"))
        .send()
        .await
        .expect("health request should succeed");

    let body = client
        .get(format!("{base}/metrics"))
        .send()
        .await
        .expect("metrics request should succeed")
        .text()
        .await
        .expect("should read body");

    assert!(
        body.contains("http_requests_total"),
        "should contain request counter:\n{body}"
    );
}

#[tokio::test]
async fn metrics_body_contains_duration_histogram() {
    let base = common::spawn_server().await;
    let client = reqwest::Client::new();

    // Make a request so the histogram has data.
    client
        .get(format!("{base}/health"))
        .send()
        .await
        .expect("health request should succeed");

    let body = client
        .get(format!("{base}/metrics"))
        .send()
        .await
        .expect("metrics request should succeed")
        .text()
        .await
        .expect("should read body");

    assert!(
        body.contains("http_request_duration_seconds"),
        "should contain duration histogram:\n{body}"
    );
}

#[tokio::test]
async fn metrics_request_counter_increments() {
    let base = common::spawn_server().await;
    let client = reqwest::Client::new();

    // Prime the counter.
    client
        .get(format!("{base}/health"))
        .send()
        .await
        .expect("health request should succeed");
    let body1 = client
        .get(format!("{base}/metrics"))
        .send()
        .await
        .expect("metrics request should succeed")
        .text()
        .await
        .expect("should read body");
    let count1 = extract_counter(&body1, "http_requests_total", "/health");

    // Make more requests.
    for _ in 0..3 {
        client
            .get(format!("{base}/health"))
            .send()
            .await
            .expect("health request should succeed");
    }

    let body2 = client
        .get(format!("{base}/metrics"))
        .send()
        .await
        .expect("metrics request should succeed")
        .text()
        .await
        .expect("should read body");
    let count2 = extract_counter(&body2, "http_requests_total", "/health");

    assert!(
        count2 > count1,
        "request counter should increment: {count1} -> {count2}"
    );
}

#[tokio::test]
async fn metrics_endpoint_excluded_from_counters() {
    let base = common::spawn_server().await;
    let client = reqwest::Client::new();

    // Scrape a few times.
    for _ in 0..3 {
        client
            .get(format!("{base}/metrics"))
            .send()
            .await
            .expect("metrics request should succeed");
    }

    let body = client
        .get(format!("{base}/metrics"))
        .send()
        .await
        .expect("metrics request should succeed")
        .text()
        .await
        .expect("should read body");

    assert!(
        !body.contains("path=\"/metrics\""),
        "/metrics should not appear in http_requests_total labels:\n{body}"
    );
}

// -- Business metrics --

#[tokio::test]
async fn metrics_body_contains_schema_count_gauge() {
    let base = common::spawn_server().await;
    let client = reqwest::Client::new();

    let body = client
        .get(format!("{base}/metrics"))
        .send()
        .await
        .expect("metrics request should succeed")
        .text()
        .await
        .expect("should read body");

    assert!(
        body.contains("kora_schema_count"),
        "should contain schema count gauge:\n{body}"
    );
}

#[tokio::test]
async fn metrics_body_contains_db_connection_gauges() {
    let base = common::spawn_server().await;
    let client = reqwest::Client::new();

    let body = client
        .get(format!("{base}/metrics"))
        .send()
        .await
        .expect("metrics request should succeed")
        .text()
        .await
        .expect("should read body");

    assert!(
        body.contains("kora_db_connections_in_use"),
        "should contain in-use connections gauge:\n{body}"
    );
    assert!(
        body.contains("kora_db_connections_idle"),
        "should contain idle connections gauge:\n{body}"
    );
}

#[tokio::test]
async fn metrics_schema_count_reflects_registrations() {
    let base = common::spawn_server().await;
    let client = reqwest::Client::new();

    // Baseline.
    let body_before = client
        .get(format!("{base}/metrics"))
        .send()
        .await
        .expect("metrics request should succeed")
        .text()
        .await
        .expect("should read body");
    let count_before = extract_gauge(&body_before, "kora_schema_count");

    // Register a unique schema under a unique subject.
    let schema = common::unique_avro_schema();
    let subject = format!("metrics-test-{}", uuid::Uuid::new_v4().as_simple());
    common::api::register_schema(&client, &base, &subject, &schema).await;

    // Updated count.
    let body_after = client
        .get(format!("{base}/metrics"))
        .send()
        .await
        .expect("metrics request should succeed")
        .text()
        .await
        .expect("should read body");
    let count_after = extract_gauge(&body_after, "kora_schema_count");

    assert!(
        count_after > count_before,
        "schema count should increase after registration: {count_before} -> {count_after}"
    );
}

// -- Helpers --

/// Extract a gauge value from Prometheus text output.
fn extract_gauge(body: &str, name: &str) -> f64 {
    for line in body.lines() {
        if line.starts_with(name) && !line.starts_with('#') {
            let value_str = line.rsplit(' ').next().unwrap_or("0");
            return value_str.parse().unwrap_or(0.0);
        }
    }
    0.0
}

/// Extract a counter value matching a specific path label.
fn extract_counter(body: &str, name: &str, path: &str) -> f64 {
    for line in body.lines() {
        if line.starts_with(name) && !line.starts_with('#') && line.contains(path) {
            let value_str = line.rsplit(' ').next().unwrap_or("0");
            return value_str.parse().unwrap_or(0.0);
        }
    }
    0.0
}
