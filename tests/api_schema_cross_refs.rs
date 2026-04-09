//! Integration tests for schema ID cross-references (Story 2.3).
//!
//! `GET /schemas/ids/{id}/subjects` — list subjects using a schema ID.
//! `GET /schemas/ids/{id}/versions` — list subject-version pairs using a schema ID.

mod common;

use reqwest::StatusCode;

// -- GET /schemas/ids/{id}/subjects --

#[tokio::test]
async fn get_subjects_by_schema_id_returns_subject() {
    let base = common::spawn_server().await;
    let client = reqwest::Client::new();
    let subject = format!("xref-subj-{}", uuid::Uuid::new_v4());

    let id = common::api::register_schema(&client, &base, &subject, common::AVRO_SCHEMA_V1).await;

    let resp = common::api::get_subjects_by_schema_id(&client, &base, id).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: Vec<String> = resp.json().await.unwrap();
    assert_eq!(body, vec![subject]);
}

#[tokio::test]
async fn get_subjects_by_nonexistent_schema_id_returns_404() {
    let base = common::spawn_server().await;
    let client = reqwest::Client::new();

    let resp = common::api::get_subjects_by_schema_id(&client, &base, i64::MAX).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error_code"], 40403);
}

#[tokio::test]
async fn get_subjects_excludes_soft_deleted_subject() {
    let base = common::spawn_server().await;
    let client = reqwest::Client::new();
    let subject = format!("xref-sdel-{}", uuid::Uuid::new_v4());

    let id = common::api::register_schema(&client, &base, &subject, common::AVRO_SCHEMA_V1).await;
    common::api::delete_subject(&client, &base, &subject).await;

    let resp = common::api::get_subjects_by_schema_id(&client, &base, id).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: Vec<String> = resp.json().await.unwrap();
    assert!(body.is_empty());
}

#[tokio::test]
async fn get_subjects_excludes_soft_deleted_version() {
    let base = common::spawn_server().await;
    let client = reqwest::Client::new();
    let subject = format!("xref-vdel-{}", uuid::Uuid::new_v4());

    let id = common::api::register_schema(&client, &base, &subject, common::AVRO_SCHEMA_V1).await;
    common::api::delete_version(&client, &base, &subject, "1").await;

    let resp = common::api::get_subjects_by_schema_id(&client, &base, id).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: Vec<String> = resp.json().await.unwrap();
    assert!(body.is_empty());
}

// -- GET /schemas/ids/{id}/versions --

#[tokio::test]
async fn get_versions_by_schema_id_returns_subject_version_pair() {
    let base = common::spawn_server().await;
    let client = reqwest::Client::new();
    let subject = format!("xref-ver-{}", uuid::Uuid::new_v4());

    let id = common::api::register_schema(&client, &base, &subject, common::AVRO_SCHEMA_V1).await;

    let resp = common::api::get_versions_by_schema_id(&client, &base, id).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: Vec<serde_json::Value> = resp.json().await.unwrap();
    assert_eq!(body.len(), 1);
    assert_eq!(body[0]["subject"], subject);
    assert_eq!(body[0]["version"], 1);
}

#[tokio::test]
async fn get_versions_by_schema_id_with_multiple_versions() {
    let base = common::spawn_server().await;
    let client = reqwest::Client::new();
    let subject = format!("xref-multi-{}", uuid::Uuid::new_v4());

    // Register two different schemas → v1 and v2 under same subject.
    let id1 = common::api::register_schema(&client, &base, &subject, common::AVRO_SCHEMA_V1).await;
    let id2 = common::api::register_schema(&client, &base, &subject, common::AVRO_SCHEMA_V2).await;

    // Each ID maps to its own version.
    let resp = common::api::get_versions_by_schema_id(&client, &base, id1).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Vec<serde_json::Value> = resp.json().await.unwrap();
    assert_eq!(body.len(), 1);
    assert_eq!(body[0]["subject"], subject);
    assert_eq!(body[0]["version"], 1);

    let resp2 = common::api::get_versions_by_schema_id(&client, &base, id2).await;
    assert_eq!(resp2.status(), StatusCode::OK);
    let body2: Vec<serde_json::Value> = resp2.json().await.unwrap();
    assert_eq!(body2.len(), 1);
    assert_eq!(body2[0]["subject"], subject);
    assert_eq!(body2[0]["version"], 2);
}

#[tokio::test]
async fn get_versions_by_nonexistent_schema_id_returns_404() {
    let base = common::spawn_server().await;
    let client = reqwest::Client::new();

    let resp = common::api::get_versions_by_schema_id(&client, &base, i64::MAX).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error_code"], 40403);
}

#[tokio::test]
async fn get_versions_excludes_soft_deleted() {
    let base = common::spawn_server().await;
    let client = reqwest::Client::new();
    let subject = format!("xref-verdel-{}", uuid::Uuid::new_v4());

    let id = common::api::register_schema(&client, &base, &subject, common::AVRO_SCHEMA_V1).await;
    common::api::delete_version(&client, &base, &subject, "1").await;

    let resp = common::api::get_versions_by_schema_id(&client, &base, id).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: Vec<serde_json::Value> = resp.json().await.unwrap();
    assert!(body.is_empty());
}

#[tokio::test]
async fn get_versions_schema_exists_but_all_deleted_returns_empty() {
    let base = common::spawn_server().await;
    let client = reqwest::Client::new();
    let subject = format!("xref-allempty-{}", uuid::Uuid::new_v4());

    let id = common::api::register_schema(&client, &base, &subject, common::AVRO_SCHEMA_V1).await;
    common::api::delete_subject(&client, &base, &subject).await;

    // Schema ID still exists (IDs are permanent), but all usages soft-deleted → 200 + []
    let resp = common::api::get_versions_by_schema_id(&client, &base, id).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: Vec<serde_json::Value> = resp.json().await.unwrap();
    assert!(body.is_empty());
}
