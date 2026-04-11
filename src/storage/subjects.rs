//! Subject storage operations.

use sqlx::PgPool;

// -- Queries --

/// List subject names, sorted alphabetically, with pagination.
///
/// When `include_deleted` is false, returns only active subjects.
/// When true, returns all subjects (active + soft-deleted).
/// `offset` defaults to 0, `limit` of -1 means unlimited.
///
/// # Errors
///
/// Returns a database error on connection failure.
pub async fn list_subjects(
    pool: &PgPool,
    include_deleted: bool,
    offset: i64,
    limit: i64,
) -> Result<Vec<String>, sqlx::Error> {
    if limit >= 0 {
        sqlx::query_scalar::<_, String>(
            "SELECT name FROM subjects WHERE deleted = false OR $1 ORDER BY name OFFSET $2 LIMIT $3",
        )
        .bind(include_deleted)
        .bind(offset)
        .bind(limit)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_scalar::<_, String>(
            "SELECT name FROM subjects WHERE deleted = false OR $1 ORDER BY name OFFSET $2",
        )
        .bind(include_deleted)
        .bind(offset)
        .fetch_all(pool)
        .await
    }
}

/// Soft-delete a subject and all its schema versions. Returns the deleted version
/// numbers sorted ascending. Runs in a transaction for consistency.
///
/// # Errors
///
/// Returns a database error on connection or transaction failure.
pub async fn soft_delete_subject(pool: &PgPool, name: &str) -> Result<Vec<i32>, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let mut versions = sqlx::query_scalar::<_, i32>(
        r"UPDATE schemas SET deleted = true
           WHERE subject_id = (SELECT id FROM subjects WHERE name = $1) AND deleted = false
           RETURNING version",
    )
    .bind(name)
    .fetch_all(&mut *tx)
    .await?;

    sqlx::query("UPDATE subjects SET deleted = true WHERE name = $1 AND deleted = false")
        .bind(name)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    versions.sort_unstable();
    Ok(versions)
}

/// Hard-delete a soft-deleted subject and all its schemas. Returns the deleted
/// version numbers sorted ascending. Runs in a transaction.
///
/// Only operates on rows where `deleted = true` (must be soft-deleted first).
///
/// # Errors
///
/// Returns a database error on connection or transaction failure.
pub async fn hard_delete_subject(pool: &PgPool, name: &str) -> Result<Vec<i32>, sqlx::Error> {
    let mut tx = pool.begin().await?;

    // Clean up schema_references for schemas being deleted (FK constraint).
    sqlx::query(
        r"DELETE FROM schema_references
           WHERE schema_id IN (
             SELECT id FROM schemas
             WHERE subject_id = (SELECT id FROM subjects WHERE name = $1) AND deleted = true
           )",
    )
    .bind(name)
    .execute(&mut *tx)
    .await?;

    let mut versions = sqlx::query_scalar::<_, i32>(
        r"DELETE FROM schemas
           WHERE subject_id = (SELECT id FROM subjects WHERE name = $1) AND deleted = true
           RETURNING version",
    )
    .bind(name)
    .fetch_all(&mut *tx)
    .await?;

    sqlx::query("DELETE FROM subjects WHERE name = $1 AND deleted = true")
        .bind(name)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    versions.sort_unstable();
    Ok(versions)
}

/// Find a subject's ID by name. Returns `None` if the subject doesn't exist or is soft-deleted.
///
/// # Errors
///
/// Returns a database error on connection failure.
pub async fn find_subject_id_by_name(pool: &PgPool, name: &str) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar::<_, i64>(
        "SELECT id FROM subjects WHERE name = $1 AND deleted = false",
    )
    .bind(name)
    .fetch_optional(pool)
    .await
}

/// Check if a subject exists by name (active only).
///
/// # Errors
///
/// Returns a database error on connection failure.
pub async fn subject_exists(pool: &PgPool, name: &str) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM subjects WHERE name = $1 AND deleted = false)",
    )
    .bind(name)
    .fetch_one(pool)
    .await
}

/// Check if a subject is soft-deleted.
///
/// Returns true if subject exists AND is soft-deleted.
///
/// # Errors
///
/// Returns a database error on connection failure.
pub async fn subject_is_soft_deleted(pool: &PgPool, name: &str) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM subjects WHERE name = $1 AND deleted = true)",
    )
    .bind(name)
    .fetch_one(pool)
    .await
}

/// Check if a subject exists by name (including soft-deleted).
///
/// # Errors
///
/// Returns a database error on connection failure.
pub async fn subject_exists_any(pool: &PgPool, name: &str) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM subjects WHERE name = $1)",
    )
    .bind(name)
    .fetch_one(pool)
    .await
}
