//! Compatibility configuration storage operations.

use sqlx::PgPool;

// -- Queries --

/// Get the per-subject compatibility level only (no fallback).
///
/// Returns `None` if no per-subject config exists.
///
/// # Errors
///
/// Returns a database error on connection failure.
pub async fn get_subject_level(pool: &PgPool, subject: &str) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar::<_, String>(
        "SELECT compatibility_level FROM config WHERE subject = $1",
    )
    .bind(subject)
    .fetch_optional(pool)
    .await
}

/// Get the global compatibility level.
///
/// # Errors
///
/// Returns a database error on connection failure.
pub async fn get_global_level(pool: &PgPool) -> Result<String, sqlx::Error> {
    sqlx::query_scalar::<_, String>(
        "SELECT compatibility_level FROM config WHERE subject IS NULL",
    )
    .fetch_one(pool)
    .await
}

/// Update the global compatibility level and return the new value.
///
/// # Errors
///
/// Returns a database error on connection failure.
pub async fn set_global_level(pool: &PgPool, level: &str) -> Result<String, sqlx::Error> {
    sqlx::query_scalar::<_, String>(
        r"UPDATE config SET compatibility_level = $1, updated_at = now()
          WHERE subject IS NULL
          RETURNING compatibility_level",
    )
    .bind(level)
    .fetch_one(pool)
    .await
}

/// Set the per-subject compatibility level (upsert). Returns the new value.
///
/// # Errors
///
/// Returns a database error on connection failure.
pub async fn set_subject_level(
    pool: &PgPool,
    subject: &str,
    level: &str,
) -> Result<String, sqlx::Error> {
    sqlx::query_scalar::<_, String>(
        r"INSERT INTO config (subject, compatibility_level)
          VALUES ($1, $2)
          ON CONFLICT (subject) DO UPDATE SET compatibility_level = $2, updated_at = now()
          RETURNING compatibility_level",
    )
    .bind(subject)
    .bind(level)
    .fetch_one(pool)
    .await
}

/// Delete per-subject config, returning the **previous** level that was deleted.
///
/// Returns `None` if no per-subject config existed.
///
/// # Errors
///
/// Returns a database error on connection failure.
pub async fn delete_subject_level(pool: &PgPool, subject: &str) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar::<_, String>(
        "DELETE FROM config WHERE subject = $1 RETURNING compatibility_level",
    )
    .bind(subject)
    .fetch_optional(pool)
    .await
}

/// Delete (reset) the global compatibility level to BACKWARD (default).
///
/// Returns the **previous** global level before the reset.
///
/// # Errors
///
/// Returns a database error on connection failure.
pub async fn delete_global_level(pool: &PgPool) -> Result<String, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let previous = sqlx::query_scalar::<_, String>(
        "SELECT compatibility_level FROM config WHERE subject IS NULL FOR UPDATE",
    )
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query(
        "UPDATE config SET compatibility_level = 'BACKWARD', updated_at = now() WHERE subject IS NULL",
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(previous)
}
