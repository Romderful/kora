//! Integration tests for checking schema registration
//! (POST /subjects/{subject}).

mod common;

use reqwest::StatusCode;

const VALID_AVRO: &str = r#"{"type":"record","name":"Test","fields":[{"name":"id","type":"int"}]}"#;
const OTHER_AVRO: &str = r#"{"type":"record","name":"Other","fields":[{"name":"x","type":"string"}]}"#;

#[tokio::test]
async fn check_registered_schema_returns_200() {
    let base = common::spawn_server().await;
    let client = reqwest::Client::new();
    let subject = format!("check-{}", uuid::Uuid::new_v4());

    // Register first.
    let reg = client
        .post(format!("{base}/subjects/{subject}/versions"))
        .json(&serde_json::json!({"schema": VALID_AVRO}))
        .send()
        .await
        .unwrap();
    let reg_body: serde_json::Value = reg.json().await.unwrap();
    let id = reg_body["id"].as_i64().unwrap();

    // Check.
    let resp = client
        .post(format!("{base}/subjects/{subject}"))
        .json(&serde_json::json!({"schema": VALID_AVRO}))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["subject"], subject);
    assert_eq!(body["id"], id);
    assert_eq!(body["version"], 1);
    assert_eq!(body["schema"], VALID_AVRO);
    assert_eq!(body["schemaType"], "AVRO");
}

#[tokio::test]
async fn check_unregistered_schema_returns_40403() {
    let base = common::spawn_server().await;
    let client = reqwest::Client::new();
    let subject = format!("check-miss-{}", uuid::Uuid::new_v4());

    // Register one schema.
    client
        .post(format!("{base}/subjects/{subject}/versions"))
        .json(&serde_json::json!({"schema": VALID_AVRO}))
        .send()
        .await
        .unwrap();

    // Check a different schema.
    let resp = client
        .post(format!("{base}/subjects/{subject}"))
        .json(&serde_json::json!({"schema": OTHER_AVRO}))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error_code"], 40403);
}

#[tokio::test]
async fn check_on_nonexistent_subject_returns_40401() {
    let base = common::spawn_server().await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{base}/subjects/nonexistent"))
        .json(&serde_json::json!({"schema": VALID_AVRO}))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error_code"], 40401);
}
