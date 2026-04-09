//! Tests for Confluent-compatible error responses.

use axum::body::to_bytes;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use kora::error::KoraError;

async fn error_response(error: KoraError) -> (StatusCode, serde_json::Value) {
    let response = error.into_response();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 1024).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    (status, body)
}

#[tokio::test]
async fn error_invalid_schema_returns_42201() {
    let (status, body) = error_response(KoraError::InvalidSchema("bad field".into())).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error_code"], 42201);
    assert!(body["message"].as_str().unwrap().contains("bad field"));
}

#[tokio::test]
async fn error_backend_data_store_returns_50001() {
    let (status, body) = error_response(KoraError::BackendDataStore("test".into())).await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(body["error_code"], 50001);
}

#[tokio::test]
async fn error_subject_not_found_returns_40401() {
    let (status, body) = error_response(KoraError::SubjectNotFound).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error_code"], 40401);
    assert_eq!(body["message"], "Subject not found");
}

#[tokio::test]
async fn error_version_not_found_returns_40402() {
    let (status, body) = error_response(KoraError::VersionNotFound).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error_code"], 40402);
    assert_eq!(body["message"], "Version not found");
}

#[tokio::test]
async fn error_schema_not_found_returns_40403() {
    let (status, body) = error_response(KoraError::SchemaNotFound).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error_code"], 40403);
    assert_eq!(body["message"], "Schema not found");
}
