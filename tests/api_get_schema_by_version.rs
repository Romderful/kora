//! Integration tests for schema retrieval by subject and version
//! (GET /subjects/{subject}/versions/{version}).

mod common;

use reqwest::StatusCode;

const AVRO_V1: &str = r#"{"type":"record","name":"Test","fields":[{"name":"id","type":"int"}]}"#;
const AVRO_V2: &str =
    r#"{"type":"record","name":"Test","fields":[{"name":"id","type":"int"},{"name":"name","type":"string"}]}"#;

async fn register(client: &reqwest::Client, base: &str, subject: &str, schema: &str) -> i64 {
    let resp = client
        .post(format!("{base}/subjects/{subject}/versions"))
        .json(&serde_json::json!({"schema": schema}))
        .send()
        .await
        .unwrap();
    resp.json::<serde_json::Value>().await.unwrap()["id"]
        .as_i64()
        .unwrap()
}

#[tokio::test]
async fn get_specific_version() {
    let base = common::spawn_server().await;
    let client = reqwest::Client::new();
    let subject = format!("ver-{}", uuid::Uuid::new_v4());

    let id1 = register(&client, &base, &subject, AVRO_V1).await;
    register(&client, &base, &subject, AVRO_V2).await;

    let resp = client
        .get(format!("{base}/subjects/{subject}/versions/1"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["subject"], subject);
    assert_eq!(body["id"], id1);
    assert_eq!(body["version"], 1);
    assert_eq!(body["schema"], AVRO_V1);
    assert_eq!(body["schemaType"], "AVRO");
}

#[tokio::test]
async fn get_latest_version() {
    let base = common::spawn_server().await;
    let client = reqwest::Client::new();
    let subject = format!("latest-{}", uuid::Uuid::new_v4());

    register(&client, &base, &subject, AVRO_V1).await;
    let id2 = register(&client, &base, &subject, AVRO_V2).await;

    let resp = client
        .get(format!("{base}/subjects/{subject}/versions/latest"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["id"], id2);
    assert_eq!(body["version"], 2);
    assert_eq!(body["schema"], AVRO_V2);
}

#[tokio::test]
async fn unknown_subject_returns_40401() {
    let base = common::spawn_server().await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{base}/subjects/nonexistent/versions/1"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error_code"], 40401);
}

#[tokio::test]
async fn unknown_version_returns_40402() {
    let base = common::spawn_server().await;
    let client = reqwest::Client::new();
    let subject = format!("ver404-{}", uuid::Uuid::new_v4());

    register(&client, &base, &subject, AVRO_V1).await;

    let resp = client
        .get(format!("{base}/subjects/{subject}/versions/99"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error_code"], 40402);
}

#[tokio::test]
async fn negative_version_returns_40402() {
    let base = common::spawn_server().await;
    let client = reqwest::Client::new();
    let subject = format!("neg-{}", uuid::Uuid::new_v4());

    register(&client, &base, &subject, AVRO_V1).await;

    let resp = client
        .get(format!("{base}/subjects/{subject}/versions/-1"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error_code"], 40402);
}

#[tokio::test]
async fn zero_version_returns_40402() {
    let base = common::spawn_server().await;
    let client = reqwest::Client::new();
    let subject = format!("zero-{}", uuid::Uuid::new_v4());

    register(&client, &base, &subject, AVRO_V1).await;

    let resp = client
        .get(format!("{base}/subjects/{subject}/versions/0"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error_code"], 40402);
}

#[tokio::test]
async fn all_schemas_soft_deleted_latest_returns_40402() {
    let base = common::spawn_server().await;
    let pool = common::pool().await;
    let client = reqwest::Client::new();
    let subject = format!("softdel-{}", uuid::Uuid::new_v4());

    let id = register(&client, &base, &subject, AVRO_V1).await;

    // Soft-delete the only schema.
    sqlx::query("UPDATE schemas SET deleted = true WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .unwrap();

    let resp = client
        .get(format!("{base}/subjects/{subject}/versions/latest"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error_code"], 40402);
}
