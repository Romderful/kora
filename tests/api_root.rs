//! Integration tests for the root endpoint (GET /, POST /).

mod common;

use reqwest::{Client, StatusCode};

#[tokio::test]
async fn get_root_returns_empty_json_object() {
    let base = common::spawn_server().await;
    let client = Client::new();

    let resp = client.get(&base).send().await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body, serde_json::json!({}));
}

#[tokio::test]
async fn post_root_returns_empty_json_object() {
    let base = common::spawn_server().await;
    let client = Client::new();

    let resp = client
        .post(&base)
        .json(&serde_json::json!({}))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body, serde_json::json!({}));
}
