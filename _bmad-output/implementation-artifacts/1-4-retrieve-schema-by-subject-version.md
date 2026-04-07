# Story 1.4: Retrieve Schema by Subject and Version

Status: done

## Story

As a **developer**,
I want to retrieve a schema by subject name and version number (or "latest"),
so that I can inspect specific versions or always get the most recent schema.

## Acceptance Criteria

1. **Given** subject "orders-value" with versions 1, 2, 3
   **When** I send `GET /subjects/orders-value/versions/2`
   **Then** I receive HTTP 200 with `{"subject": "orders-value", "id": <id>, "version": 2, "schema": "<json>", "schemaType": "AVRO"}`

2. **Given** subject "orders-value" with 3 versions
   **When** I send `GET /subjects/orders-value/versions/latest`
   **Then** I receive the version 3 schema

3. **Given** a non-existent subject
   **When** I send `GET /subjects/unknown/versions/1`
   **Then** I receive HTTP 404 with Confluent error code 40401

4. **Given** a valid subject but non-existent version
   **When** I send `GET /subjects/orders-value/versions/99`
   **Then** I receive HTTP 404 with Confluent error code 40402

**FRs Covered:** FR3, FR4

## Tasks / Subtasks

- [x] Task 1: Add error variants `SubjectNotFound` and `VersionNotFound` (AC: #3, #4)
  - [x] Add `SubjectNotFound` variant to `KoraError` — maps to 40401 / 404
  - [x] Add `VersionNotFound` variant to `KoraError` — maps to 40402 / 404
  - [x] Add `status_code()` and `error_code()` match arms
  - [x] Unit tests in `tests/error_responses.rs`: both errors serialize correctly

- [x] Task 2: Storage layer — find schema by subject + version (AC: #1, #2)
  - [x] Add `subjects::exists(pool, name) -> Result<bool, sqlx::Error>` in `src/storage/subjects.rs`
  - [x] Add `schemas::find_by_subject_version(pool, subject, version) -> Result<Option<SchemaRow>, sqlx::Error>` in `src/storage/schemas.rs`
  - [x] Add `schemas::find_latest_by_subject(pool, subject) -> Result<Option<SchemaRow>, sqlx::Error>` in `src/storage/schemas.rs`
  - [x] Define `SchemaRow` struct: `{ id, subject, version, schema_type, schema }`
  - [x] SQL joins `schemas` + `subjects`, filters `deleted = false` (unlike GET by ID, version-based retrieval respects soft-delete)

- [x] Task 3: API handler — GET /subjects/{subject}/versions/{version} (AC: #1, #2, #3, #4)
  - [x] Add `get_schema_by_version` handler in `src/api/subjects.rs`
  - [x] Path parameters: `(subject, version)` as `(String, String)` — version is String to support "latest"
  - [x] Check subject existence first → `SubjectNotFound` (40401) if missing
  - [x] Parse version: "latest" → `find_latest_by_subject`, numeric → `find_by_subject_version`
  - [x] Non-numeric, non-"latest" version → `VersionNotFound` (40402)
  - [x] Return Confluent response: `{"subject", "id", "version", "schema", "schemaType"}`
  - [x] Register route in `src/api/mod.rs`: `.route("/subjects/{subject}/versions/{version}", get(...))`

- [x] Task 4: Integration tests (AC: #1, #2, #3, #4)
  - [x] Create `tests/api_get_schema_by_version.rs`
  - [x] Test: register 2 versions, GET version 1 → 200 with correct version data (AC #1)
  - [x] Test: register 2 versions, GET "latest" → 200 with version 2 data (AC #2)
  - [x] Test: GET non-existent subject → 404 + error code 40401 (AC #3)
  - [x] Test: GET valid subject but non-existent version → 404 + error code 40402 (AC #4)

- [x] Task 5: Verify all tests pass
  - [x] `just lint` — zero warnings
  - [x] `just test` — 36/36 tests pass (30 existing + 6 new)

## Dev Notes

### Confluent API Contract

```
GET /subjects/{subject}/versions/{version}
Accept: application/vnd.schemaregistry.v1+json

Response (200):
{
  "subject": "orders-value",
  "id": 1,
  "version": 2,
  "schema": "{\"type\":\"record\",...}",
  "schemaType": "AVRO"
}

Response (404 - subject not found):
{"error_code": 40401, "message": "Subject not found"}

Response (404 - version not found):
{"error_code": 40402, "message": "Version not found"}
```

The `version` path parameter accepts either an integer or the string `"latest"`.

### Error Codes

- `40401` = Subject not found (HTTP 404) — subject name doesn't exist in `subjects` table
- `40402` = Version not found (HTTP 404) — subject exists but no schema at that version
- Distinct from `40403` = Schema not found (used by GET /schemas/ids/{id})

### Soft-Delete Behavior (CRITICAL DIFFERENCE FROM 1.3)

Unlike `GET /schemas/ids/{id}` which ignores soft-delete (IDs are permanent), version-based retrieval **respects** soft-delete: `WHERE deleted = false`. This matches Confluent behavior — soft-deleted schemas disappear from version listings but remain resolvable by global ID.

### Architecture Compliance

- **Handler pattern**: `async fn handler(State(pool), Path((subject, version))) -> Result<impl IntoResponse, KoraError>`
- **Storage pattern**: Standalone async functions taking `&PgPool` + query params. Use `sqlx::query()` (runtime, no macros) with `sqlx::Row` for multi-column results.
- **File placement**: Handler in existing `src/api/subjects.rs` (same resource). Storage in existing `src/storage/schemas.rs` and `src/storage/subjects.rs`.
- **Response struct**: New `GetSchemaByVersionResponse` with `#[serde(rename = "schemaType")]` for Confluent wire format.
- **No `unwrap()`** outside tests. `deny(missing_docs)` + `deny(clippy::pedantic)` enforced.

### SQL Queries

**Find by subject + version:**
```sql
SELECT s.id, sub.name as subject, s.version, s.schema_type, s.schema_text
FROM schemas s JOIN subjects sub ON s.subject_id = sub.id
WHERE sub.name = $1 AND s.version = $2 AND s.deleted = false
```

**Find latest by subject:**
```sql
SELECT s.id, sub.name as subject, s.version, s.schema_type, s.schema_text
FROM schemas s JOIN subjects sub ON s.subject_id = sub.id
WHERE sub.name = $1 AND s.deleted = false
ORDER BY s.version DESC LIMIT 1
```

**Check subject exists:**
```sql
SELECT COUNT(*) FROM subjects WHERE name = $1
```

### Previous Story Intelligence (from 1.3)

- `KoraError` has 3 variants: `InvalidSchema` (42201/422), `SchemaNotFound` (40403/404), `BackendDataStore` (50001/500). Add `SubjectNotFound` (40401/404) and `VersionNotFound` (40402/404).
- Storage functions are standalone async fns taking `&PgPool` — follow same pattern.
- Integration tests use `common::spawn_server()` + `common::pool()`.
- `VALID_AVRO` constant: `r#"{"type":"record","name":"Test","fields":[{"name":"id","type":"int"}]}"#`
- Current test count: 30 tests across 7 test files.
- Content-Type middleware sets `application/vnd.schemaregistry.v1+json` globally.
- sqlx queries use runtime `query_scalar::<_, T>()` — no compile-time macros (changed in refactor commit `7b4b29b`).

### Testing Strategy

**Integration tests** (`tests/api_get_schema_by_version.rs`):
- Helper function to register schemas via POST (reused across tests)
- Register 2 different Avro schemas under same subject → creates versions 1 and 2
- GET specific version → verify all response fields (subject, id, version, schema, schemaType)
- GET "latest" → verify returns highest version
- GET non-existent subject → 404 + 40401
- GET valid subject + non-existent version → 404 + 40402

**Unit tests** (`tests/error_responses.rs`):
- `subject_not_found` → serializes to `{"error_code": 40401, "message": "Subject not found"}`
- `version_not_found` → serializes to `{"error_code": 40402, "message": "Version not found"}`

### Project Structure Notes

Files to create:
```
tests/api_get_schema_by_version.rs  ← integration tests
```

Files to modify:
```
src/error.rs                        ← add SubjectNotFound, VersionNotFound variants
src/api/mod.rs                      ← add route
src/api/subjects.rs                 ← add handler + response struct
src/storage/schemas.rs              ← add SchemaRow, find_by_subject_version, find_latest_by_subject
src/storage/subjects.rs             ← add exists()
tests/error_responses.rs            ← add unit tests for new error variants
```

### References

- [Source: epics.md — Epic 1, Story 1.4]
- [Source: prd.md — FR3, FR4]
- [Source: architecture.md — API patterns, error handling, database schema]
- [Source: 1-3-retrieve-schema-by-global-id.md — Completion Notes, Dev Notes]
- [Source: Confluent Schema Registry API — GET /subjects/{subject}/versions/{version}]

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6 (1M context)

### Debug Log References

None required.

### Completion Notes List

- Added `SubjectNotFound` (40401) and `VersionNotFound` (40402) error variants
- Added `subjects::exists()`, `schemas::find_by_subject_version()`, `schemas::find_latest_by_subject()` storage functions
- Added `SchemaRow` struct with `row_to_schema` helper for multi-column query results
- Added `get_schema_by_version` handler with "latest" support in `src/api/subjects.rs`
- Route registered: `GET /subjects/{subject}/versions/{version}`
- 4 integration tests + 2 unit tests = 6 new tests (36 total, all passing)
- `just lint` passes with zero warnings
- Fixed clippy pedantic: `r#""#` → `r""`, pass-by-ref for `row_to_schema_version`
- Review round 1: fixed TOCTOU (exists check moved to error path), added version validation (v < 1), added validate_subject to GET, optimized SELECT EXISTS, added soft-delete test
- Review round 2: clean — all findings dismissed (spec-compliant or pre-existing)
- Simplified: removed `GetSchemaByVersionResponse` — `SchemaVersion` serves as both storage and API struct
- Simplified: removed `GetSchemaResponse` — replaced with inline `serde_json::json!`
- Removed unused `Deserialize` from `ErrorBody`
- Renamed `tests/api_get_schema.rs` → `tests/api_get_schema_by_id.rs` for consistency

### File List

**New:**
- `tests/api_get_schema_by_version.rs`

**Renamed:**
- `tests/api_get_schema.rs` → `tests/api_get_schema_by_id.rs`

**Modified:**
- `src/error.rs` — added `SubjectNotFound`, `VersionNotFound` variants, removed unused `Deserialize`
- `src/api/mod.rs` — added route
- `src/api/schemas.rs` — removed `GetSchemaResponse`, use inline json
- `src/api/subjects.rs` — added `get_schema_by_version` handler
- `src/storage/schemas.rs` — added `SchemaVersion`, `find_by_subject_version`, `find_latest_by_subject`
- `src/storage/subjects.rs` — added `exists()`
- `tests/error_responses.rs` — added 2 unit tests
