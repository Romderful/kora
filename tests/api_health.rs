//! Integration tests for the health endpoint.

mod common;

use reqwest::StatusCode;

#[tokio::test]
async fn health_pg_up_returns_200() {
    let base = common::spawn_server().await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{base}/health"))
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
    assert_eq!(ct, "application/json", "default should be application/json");

    let body: serde_json::Value = resp.json().await.expect("should parse json");
    assert_eq!(body["status"], "UP");
}

#[tokio::test]
async fn health_returns_vendor_content_type_when_requested() {
    let base = common::spawn_server().await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{base}/health"))
        .header("Accept", "application/vnd.schemaregistry.v1+json")
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
    assert_eq!(ct, "application/vnd.schemaregistry.v1+json");
}
