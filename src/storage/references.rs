//! Schema reference storage operations.

use sqlx::PgPool;

use crate::types::SchemaReference;
use crate::error::KoraError;

// -- Queries --

/// Validate that all referenced schemas exist and are not soft-deleted.
///
/// # Errors
///
/// Returns `KoraError::ReferenceNotFound` if any referenced subject/version
/// does not exist or is soft-deleted.
pub async fn validate_references(
    pool: &PgPool,
    refs: &[SchemaReference],
) -> Result<(), KoraError> {
    for r in refs {
        let exists = sqlx::query_scalar::<_, bool>(
            r"SELECT EXISTS(
                SELECT 1 FROM schemas s JOIN subjects sub ON s.subject_id = sub.id
                WHERE sub.name = $1 AND s.version = $2
                  AND s.deleted = false AND sub.deleted = false
            )",
        )
        .bind(&r.subject)
        .bind(r.version)
        .fetch_one(pool)
        .await?;

        if !exists {
            return Err(KoraError::ReferenceNotFound(format!(
                "Schema reference not found: subject '{}' version {}",
                r.subject, r.version
            )));
        }
    }
    Ok(())
}

/// Find all references for a given schema ID.
///
/// # Errors
///
/// Returns a database error on connection failure.
pub async fn find_references_by_schema_id(
    pool: &PgPool,
    schema_id: i64,
) -> Result<Vec<SchemaReference>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT name, subject, version FROM schema_references WHERE schema_id = $1 ORDER BY name",
    )
    .bind(schema_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .iter()
        .map(|row| SchemaReference {
            name: sqlx::Row::get(row, "name"),
            subject: sqlx::Row::get(row, "subject"),
            version: sqlx::Row::get(row, "version"),
        })
        .collect())
}

/// Check if a subject/version is referenced by any **active** (non-deleted) schema.
///
/// Joins with the schemas table to ignore references from soft-deleted or
/// hard-deleted schemas — a deleted dependent should not block deletion of
/// its dependency.
///
/// # Errors
///
/// Returns a database error on connection failure.
pub async fn is_version_referenced(
    pool: &PgPool,
    subject: &str,
    version: i32,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar::<_, bool>(
        r"SELECT EXISTS(
            SELECT 1 FROM schema_references sr
            JOIN schemas s ON sr.schema_id = s.id
            WHERE sr.subject = $1 AND sr.version = $2
              AND s.deleted = false
        )",
    )
    .bind(subject)
    .bind(version)
    .fetch_one(pool)
    .await
}

