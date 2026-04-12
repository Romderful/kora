# Story 2.3: Schema ID Cross-References

Status: done

## Story

As a **developer**,
I want to find all subjects and versions that use a given schema ID,
so that I can understand the impact of a schema across the registry.

## Acceptance Criteria

1. **Given** schema ID 1 is used by subjects "orders-value" (v1) and "users-value" (v2)
   **When** I send `GET /schemas/ids/1/subjects`
   **Then** I receive HTTP 200 with `["orders-value", "users-value"]`

2. **Given** schema ID 1 is registered as version 1 under "orders-value"
   **When** I send `GET /schemas/ids/1/versions`
   **Then** I receive HTTP 200 with `[{"subject": "orders-value", "version": 1}]`

3. **Given** a non-existent schema ID
   **When** I send `GET /schemas/ids/999/subjects`
   **Then** I receive HTTP 404 with Confluent error code 40403

4. **Given** a non-existent schema ID
   **When** I send `GET /schemas/ids/999/versions`
   **Then** I receive HTTP 404 with Confluent error code 40403

**FRs Covered:** FR13, FR14

## Tasks / Subtasks

- [x] Task 1: Storage layer — cross-reference queries (AC: #1, #2, #3, #4)
  - [x] Add `schemas::find_subjects_by_id(pool, id) -> Result<Vec<String>, sqlx::Error>`
  - [x] Add `schemas::find_versions_by_id(pool, id) -> Result<Vec<SubjectVersion>, sqlx::Error>`
  - [x] Add `SubjectVersion` struct with `subject: String, version: i32` (serde Serialize)
  - [x] Add `schemas::exists(pool, id) -> Result<bool, sqlx::Error>` — ignores soft-delete

- [x] Task 2: API handlers — two new endpoints (AC: #1, #2, #3, #4)
  - [x] Add `schemas::get_subjects_by_schema_id` handler — GET /schemas/ids/{id}/subjects
  - [x] Add `schemas::get_versions_by_schema_id` handler — GET /schemas/ids/{id}/versions
  - [x] Both: if schema ID doesn't exist at all → SchemaNotFound (40403)
  - [x] Both: return empty array if schema exists but all usages are soft-deleted

- [x] Task 3: Route registration (AC: #1, #2)
  - [x] Add route `/schemas/ids/{id}/subjects` → GET schemas::get_subjects_by_schema_id
  - [x] Add route `/schemas/ids/{id}/versions` → GET schemas::get_versions_by_schema_id

- [x] Task 4: Integration tests (AC: #1, #2, #3, #4)
  - [x] Create `tests/api_schema_cross_refs.rs`
  - [x] Add test helpers `get_subjects_by_schema_id` and `get_versions_by_schema_id` to `tests/common/api.rs`
  - [x] Test: register schema → GET /subjects returns subject name (AC #1)
  - [x] Test: register schema → GET /versions returns subject-version pair (AC #2)
  - [x] Test: non-existent schema ID → 404 + 40403 for /subjects (AC #3)
  - [x] Test: non-existent schema ID → 404 + 40403 for /versions (AC #4)
  - [x] Test: soft-deleted subject excluded from /subjects results
  - [x] Test: soft-deleted version excluded from /subjects results
  - [x] Test: soft-deleted version excluded from /versions results
  - [x] Test: schema exists but all usages soft-deleted → 200 + empty array

- [x] Task 5: Verify all tests pass
  - [x] `cargo clippy` — zero warnings (pedantic)
  - [x] `cargo test` — 68 tests pass (60 existing + 8 new)

### Review Findings

- [x] [Review][Decision] AC1 multi-subject test — deferred to IMPORT mode (story 6.1), endpoint is correct
- [x] [Review][Patch] Remove unnecessary DISTINCT in find_subjects_by_id [src/storage/schemas.rs]
- [x] [Review][Patch] Add test for multiple versions under one subject for /versions endpoint
- [x] [Review][Defer] Test hard-delete + cross-ref lookup interaction — deferred, pre-existing

## Dev Notes

### Confluent API Contract

```
GET /schemas/ids/{id}/subjects
Response (200): ["orders-value", "users-value"]
Response (404): {"error_code": 40403, "message": "Schema not found"}

GET /schemas/ids/{id}/versions
Response (200): [{"subject": "orders-value", "version": 1}]
Response (404): {"error_code": 40403, "message": "Schema not found"}
```

### Important: Schema ID vs Fingerprint

Schema content is globally deduplicated. Two subjects registering the same schema text get the **same global content ID** (from `schema_contents`). `GET /schemas/ids/{id}/subjects` returns all subjects that have a version pointing to that content. This matches Confluent behavior where schema IDs are global across subjects.

### Soft-Delete Filtering

Both endpoints must filter out soft-deleted schemas (`s.deleted = false`) AND soft-deleted subjects (`sub.deleted = false`). However, we need `schema_exists()` to check if the ID exists at all (including soft-deleted) to distinguish between "schema doesn't exist" (404) and "schema exists but all usages are deleted" (200 + empty array). This matches `find_by_id` which ignores soft-delete because IDs are permanent.

### Existing Patterns to Follow

- **Handler pattern**: see `get_schema_by_id` in `src/api/schemas.rs` — same `State(pool) + Path(id)` extraction
- **Storage pattern**: see `find_by_id` in `src/storage/schemas.rs` — sqlx query with join
- **Route pattern**: see `api/mod.rs` — add new `.route()` calls with `get()` handler
- **Test pattern**: see `tests/api_get_schema_by_id.rs` — spawn_server + client + helpers
- **Test helpers**: see `tests/common/api.rs` — add helpers following same async pattern

### Architecture Compliance

- Handlers call storage directly (no service layer)
- `Result<impl IntoResponse, KoraError>` return type
- Error mapping: non-existent schema → `KoraError::SchemaNotFound` (40403)
- JSON response via `Json(serde_json::json!(...))` for subjects, `Json(vec)` for versions
- No cache layer yet (deferred to later epic)

### References

- [Source: epics.md — Epic 2, Story 2.3]
- [Source: prd.md — FR13, FR14]
- [Source: architecture.md — API patterns, storage patterns]
- [Source: 2-2-hard-delete-subject-and-versions.md — Completion Notes, test helpers pattern]

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6 (1M context)

### Debug Log References

### Completion Notes List

- Added `SubjectVersion` struct for cross-reference version responses
- Added `schemas::exists()` — checks schema ID existence ignoring soft-delete (IDs are permanent)
- Added `schemas::find_subjects_by_id()` — DISTINCT subject names, filters soft-deleted schemas+subjects
- Added `schemas::find_versions_by_id()` — subject-version pairs, filters soft-deleted
- Added two handlers: `get_subjects_by_schema_id`, `get_versions_by_schema_id`
- Both handlers: 404 if schema ID doesn't exist, 200+[] if all usages soft-deleted
- Registered routes `/schemas/ids/{id}/subjects` and `/schemas/ids/{id}/versions`
- 8 integration tests covering all 4 ACs + soft-delete filtering edge cases
- 68 tests total, all passing, zero clippy warnings

### File List

**New:**
- `tests/api_schema_cross_refs.rs`

**Modified:**
- `src/storage/schemas.rs` — added SubjectVersion, exists(), find_subjects_by_id(), find_versions_by_id()
- `src/api/schemas.rs` — added get_subjects_by_schema_id(), get_versions_by_schema_id()
- `src/api/mod.rs` — added two new routes
- `tests/common/api.rs` — added cross-reference test helpers

### Change Log

- 2026-04-08: Story 2.3 implemented — schema ID cross-reference endpoints (FR13, FR14)
