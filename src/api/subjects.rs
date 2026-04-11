//! Subject-related API handlers.

use axum::{
    Json,
    extract::{Path, Query, State, rejection::JsonRejection},
    response::IntoResponse,
};
use serde::Deserialize;
use sqlx::PgPool;

use crate::error::KoraError;
use crate::schema::{self, SchemaFormat};
use crate::storage::{references, schemas, subjects};
use crate::types::SchemaReference;

// -- Types --

/// Request body for schema registration and check endpoints.
#[derive(Debug, Deserialize)]
pub struct SchemaRequest {
    /// The raw schema string (JSON-encoded).
    pub schema: String,
    /// Schema format — defaults to AVRO when absent.
    #[serde(rename = "schemaType")]
    pub schema_type: Option<String>,
    /// Optional schema references (for Protobuf imports, JSON Schema `$ref`, etc.).
    #[serde(default)]
    pub references: Option<Vec<SchemaReference>>,
}

/// Query parameters for list endpoints supporting `?deleted=true` with pagination.
#[derive(Debug, Deserialize)]
pub struct ListParams {
    /// When true, include soft-deleted items in the response.
    #[serde(default)]
    pub deleted: bool,
    /// Pagination offset (default 0).
    #[serde(default)]
    pub offset: i64,
    /// Pagination limit (-1 = unlimited, default).
    #[serde(default = "default_limit")]
    pub limit: i64,
}

/// Default limit: -1 (unlimited). Shared by all pagination param structs.
pub(crate) const fn default_limit() -> i64 {
    -1
}

/// Query parameters for DELETE endpoints supporting `?permanent=true`.
#[derive(Debug, Deserialize)]
pub struct PermanentParam {
    /// When true, hard-delete (requires prior soft-delete).
    #[serde(default)]
    pub permanent: bool,
}

// -- Handlers --

/// Register a schema under a subject.
///
/// `POST /subjects/{subject}/versions`
///
/// # Errors
///
/// Returns `KoraError::InvalidSchema` (422) for unparseable schemas or
/// `KoraError::BackendDataStore` (500) for database failures.
pub async fn register_schema(
    State(pool): State<PgPool>,
    Path(subject): Path<String>,
    body: Result<Json<SchemaRequest>, JsonRejection>,
) -> Result<impl IntoResponse, KoraError> {
    let Json(body) = body.map_err(|e| KoraError::InvalidSchema(e.body_text()))?;

    validate_subject(&subject)?;

    let format = SchemaFormat::from_optional(body.schema_type.as_deref())?;
    let parsed = schema::parse(format, &body.schema)?;

    // Validate references before any writes.
    let refs = body.references.as_deref().unwrap_or_default();
    if !refs.is_empty() {
        references::validate_references(&pool, refs).await?;
    }

    let (id, _is_new) = schemas::register_schema_atomically(
        &pool,
        &subject,
        &schemas::NewSchema {
            schema_type: format.as_str(),
            schema_text: &body.schema,
            canonical_form: &parsed.canonical_form,
            fingerprint: &parsed.fingerprint,
        },
        refs,
    )
    .await?;

    Ok(Json(serde_json::json!({ "id": id })))
}

/// Check if a schema is registered under a subject.
///
/// `POST /subjects/{subject}`
///
/// # Errors
///
/// Returns `KoraError::SubjectNotFound` (40401) if the subject doesn't exist,
/// or `KoraError::SchemaNotFound` (40403) if the schema is not registered.
pub async fn check_schema(
    State(pool): State<PgPool>,
    Path(subject): Path<String>,
    body: Result<Json<SchemaRequest>, JsonRejection>,
) -> Result<impl IntoResponse, KoraError> {
    let Json(body) = body.map_err(|e| KoraError::InvalidSchema(e.body_text()))?;

    validate_subject(&subject)?;

    let format = SchemaFormat::from_optional(body.schema_type.as_deref())?;
    let parsed = schema::parse(format, &body.schema)?;

    let subject_id = subjects::find_subject_id_by_name(&pool, &subject)
        .await?
        .ok_or(KoraError::SubjectNotFound)?;

    let sv = schemas::find_schema_by_subject_id_and_fingerprint(&pool, subject_id, &parsed.fingerprint)
        .await?
        .ok_or(KoraError::SchemaNotFound)?;

    Ok(Json(sv))
}

/// List registered subjects.
///
/// `GET /subjects` — non-deleted subjects (default).
/// `GET /subjects?deleted=true` — all subjects (including soft-deleted).
///
/// # Errors
///
/// Returns `KoraError::BackendDataStore` (500) for database failures.
pub async fn list_subjects(
    State(pool): State<PgPool>,
    Query(params): Query<ListParams>,
) -> Result<impl IntoResponse, KoraError> {
    let names = subjects::list_subjects(&pool, params.deleted, params.offset.max(0), params.limit).await?;
    Ok(Json(names))
}

/// List all versions of a subject.
///
/// `GET /subjects/{subject}/versions`
///
/// # Errors
///
/// Returns `KoraError::SubjectNotFound` (40401) if the subject doesn't exist,
/// or `KoraError::BackendDataStore` (500) for database failures.
pub async fn list_versions(
    State(pool): State<PgPool>,
    Path(subject): Path<String>,
    Query(params): Query<ListParams>,
) -> Result<impl IntoResponse, KoraError> {
    validate_subject(&subject)?;

    if !subjects::subject_exists(&pool, &subject).await? {
        return Err(KoraError::SubjectNotFound);
    }

    let versions = schemas::list_schema_versions(&pool, &subject, params.deleted, params.offset.max(0), params.limit).await?;
    Ok(Json(versions))
}

/// Retrieve a schema by subject and version.
///
/// `GET /subjects/{subject}/versions/{version}`
///
/// The version can be a number or "latest".
///
/// # Errors
///
/// Returns `KoraError::SubjectNotFound` (40401) if the subject doesn't exist,
/// or `KoraError::VersionNotFound` (40402) if the version doesn't exist.
pub async fn get_schema_by_version(
    State(pool): State<PgPool>,
    Path((subject, version)): Path<(String, String)>,
) -> Result<impl IntoResponse, KoraError> {
    validate_subject(&subject)?;

    let row = if version == "latest" {
        schemas::find_latest_schema_by_subject(&pool, &subject).await?
    } else {
        let v = parse_version(&version)?;
        schemas::find_schema_by_subject_version(&pool, &subject, v).await?
    };

    match row {
        Some(sv) => Ok(Json(sv)),
        None if subjects::subject_exists(&pool, &subject).await? => Err(KoraError::VersionNotFound),
        None => Err(KoraError::SubjectNotFound),
    }
}

/// Soft-delete a subject and all its versions.
///
/// `DELETE /subjects/{subject}`
///
/// # Errors
///
/// Returns `KoraError::SubjectNotFound` (40401) if the subject doesn't exist
/// (or isn't soft-deleted when `permanent=true`).
pub async fn delete_subject(
    State(pool): State<PgPool>,
    Path(subject): Path<String>,
    Query(params): Query<PermanentParam>,
) -> Result<impl IntoResponse, KoraError> {
    validate_subject(&subject)?;

    if params.permanent {
        // Confluent requires subject to be soft-deleted first (40405).
        // subject_exists returns false for both nonexistent AND soft-deleted subjects,
        // so we must distinguish: if active → 40405, if nonexistent → let hard_delete handle 40401.
        if subjects::subject_exists(&pool, &subject).await? {
            return Err(KoraError::SubjectNotSoftDeleted(subject));
        }
        // Check if any version of this subject is referenced by other schemas.
        let versions_to_delete =
            schemas::list_schema_versions(&pool, &subject, true, 0, -1).await?;
        for v in &versions_to_delete {
            if references::is_version_referenced(&pool, &subject, *v).await? {
                return Err(KoraError::ReferenceExists(format!(
                    "{subject} version {v}"
                )));
            }
        }
        let versions = subjects::hard_delete_subject(&pool, &subject).await?;
        if versions.is_empty() {
            return Err(KoraError::SubjectNotFound);
        }
        Ok(Json(versions))
    } else {
        if !subjects::subject_exists(&pool, &subject).await? {
            return Err(subject_not_found_or_soft_deleted(&pool, &subject).await);
        }
        let versions = subjects::soft_delete_subject(&pool, &subject).await?;
        Ok(Json(versions))
    }
}

/// Delete a single schema version (soft or hard).
///
/// `DELETE /subjects/{subject}/versions/{version}`
/// `DELETE /subjects/{subject}/versions/{version}?permanent=true`
///
/// # Errors
///
/// Returns `KoraError::SubjectNotFound` (40401) or `KoraError::VersionNotFound` (40402).
pub async fn delete_version(
    State(pool): State<PgPool>,
    Path((subject, version)): Path<(String, String)>,
    Query(params): Query<PermanentParam>,
) -> Result<impl IntoResponse, KoraError> {
    validate_subject(&subject)?;

    let deleted = if params.permanent {
        let v = parse_version(&version)?;
        // Subject must exist (even soft-deleted) for hard-delete to make sense.
        if !subjects::subject_exists_any(&pool, &subject).await? {
            return Err(KoraError::SubjectNotFound);
        }
        // Confluent requires version to be soft-deleted first (40407).
        if schemas::version_is_active(&pool, &subject, v).await? {
            return Err(KoraError::SchemaVersionNotSoftDeleted(subject, v));
        }
        if references::is_version_referenced(&pool, &subject, v).await? {
            return Err(KoraError::ReferenceExists(format!(
                "{subject} version {v}"
            )));
        }
        schemas::hard_delete_schema_version(&pool, &subject, v).await?
    } else {
        if !subjects::subject_exists(&pool, &subject).await? {
            return Err(KoraError::SubjectNotFound);
        }
        if version == "latest" {
            schemas::soft_delete_latest_schema(&pool, &subject).await?
        } else {
            let v = parse_version(&version)?;
            // Return 40406 if version is already soft-deleted.
            if schemas::version_is_soft_deleted(&pool, &subject, v).await? {
                return Err(KoraError::SchemaVersionSoftDeleted(subject, v));
            }
            schemas::soft_delete_schema_version(&pool, &subject, v).await?
        }
    }
    .ok_or(KoraError::VersionNotFound)?;

    Ok(Json(deleted))
}

// -- Helpers --

/// Return `SubjectNotFound` or `SubjectSoftDeleted` based on subject state.
async fn subject_not_found_or_soft_deleted(pool: &PgPool, subject: &str) -> KoraError {
    if subjects::subject_is_soft_deleted(pool, subject).await.unwrap_or(false) {
        KoraError::SubjectSoftDeleted(subject.to_string())
    } else {
        KoraError::SubjectNotFound
    }
}

/// Maximum allowed length for a subject name.
const MAX_SUBJECT_LENGTH: usize = 255;

/// Validate the subject path parameter.
fn validate_subject(subject: &str) -> Result<(), KoraError> {
    if subject.is_empty() {
        return Err(KoraError::InvalidSchema(
            "Subject name must not be empty".into(),
        ));
    }
    if subject.len() > MAX_SUBJECT_LENGTH {
        return Err(KoraError::InvalidSchema(
            "Subject name exceeds maximum length".into(),
        ));
    }
    if subject.contains('\0') {
        return Err(KoraError::InvalidSchema(
            "Subject name contains invalid characters".into(),
        ));
    }
    Ok(())
}

/// Parse a version string to a positive i32.
///
/// Confluent accepts only positive integers and `"latest"`. Everything else
/// (0, negatives, non-numeric strings) returns 42202 (`InvalidVersion`).
fn parse_version(version: &str) -> Result<i32, KoraError> {
    let v: i32 = version
        .parse()
        .map_err(|_| KoraError::InvalidVersion(version.to_string()))?;
    if v < 1 {
        return Err(KoraError::InvalidVersion(version.to_string()));
    }
    Ok(v)
}
