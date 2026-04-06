# Story 1.3: Retrieve Schema by Global ID

Status: done

## Story

As a **developer**,
I want to retrieve a schema by its global ID,
so that my deserializers can resolve schemas from the ID embedded in Kafka messages.

## Acceptance Criteria

1. **Given** a registered schema with ID N, **When** I send `GET /schemas/ids/{N}`, **Then** I receive HTTP 200 with `{"schema": "<schema_json>"}`
2. **Given** a non-existent schema ID, **When** I send `GET /schemas/ids/999`, **Then** I receive HTTP 404 with Confluent error code 40403 (`{"error_code": 40403, "message": "Schema not found"}`)
3. **Given** a soft-deleted schema, **When** I send `GET /schemas/ids/{N}`, **Then** I still receive HTTP 200 (schemas are retrievable by global ID even after soft-delete — Confluent behavior)

## Tasks / Subtasks

- [x] Task 1: Add `SchemaNotFound` error variant (AC: #2)
  - [x] Add `SchemaNotFound` unit variant to `KoraError` — maps to 40403 / 404
  - [x] Add `status_code()` and `error_code()` match arms
  - [x] Unit test: error serializes to `{"error_code": 40403, "message": "Schema not found"}`

- [x] Task 2: Storage layer — find schema by ID (AC: #1, #3)
  - [x] Add `find_by_id(pool, id) -> Result<Option<String>, sqlx::Error>` in `src/storage/schemas.rs`
  - [x] SQL: `SELECT schema_text FROM schemas WHERE id = $1` (NO `deleted = false` filter — Confluent returns schemas by global ID even if soft-deleted)

- [x] Task 3: API handler — GET /schemas/ids/{id} (AC: #1, #2)
  - [x] Create `src/api/schemas.rs` with `get_schema_by_id` handler
  - [x] Path parameter: `id` as `i64` (axum `Path(id): Path<i64>`)
  - [x] On found: return `{"schema": "<schema_text>"}` with HTTP 200
  - [x] On not found: return `KoraError::SchemaNotFound`
  - [x] Register route in `src/api/mod.rs`: `.route("/schemas/ids/{id}", get(schemas::get_schema_by_id))`
  - [x] Add `pub mod schemas;` in `src/api/mod.rs`

- [x] Task 4: Integration tests (AC: #1, #2, #3)
  - [x] Create `tests/api_get_schema.rs`
  - [x] Test: register a schema, then GET by returned ID → 200 + `{"schema": VALID_AVRO}`
  - [x] Test: GET non-existent ID (e.g., `i64::MAX`) → 404 + `{"error_code": 40403}`
  - [x] Test: GET with invalid path (e.g., `/schemas/ids/abc`) → 400 (axum path rejection)
  - [x] Test: soft-deleted schema still returns 200 (AC #3 — review finding)

- [x] Task 5: Verify CI-readiness (AC: #1-#3)
  - [x] `make lint` — zero warnings
  - [x] `make test` — 30/30 pass

## Dev Notes

### Confluent API Contract

```
GET /schemas/ids/{id}
Accept: application/vnd.schemaregistry.v1+json

Response (200): {"schema": "<schema_json_string>"}
Response (404): {"error_code": 40403, "message": "Schema not found"}
Response (500): {"error_code": 50001, "message": "Error in the backend data store"}
```

**Key behaviors from Confluent docs:**
- The response only contains `{"schema": "..."}` — just the schema string, nothing else
- The schema is returned as a JSON-encoded string (the raw `schema_text` as stored)
- Schemas are retrievable by global ID even after soft-delete (the ID is permanent)
- Invalid ID format returns 400 (axum handles this via Path rejection)
- The `format` query parameter exists in Confluent (for resolved/serialized) but is NOT needed for MVP — just return the raw schema text

### Error Code: 40403

From Confluent API spec, error code `40403` = "Schema not found". This is distinct from:
- `40401` = Subject not found
- `40402` = Version not found

The first three digits map to HTTP status (404), the last two are the sub-code.

### Architecture Compliance

- **Handler pattern**: `async fn handler(State(pool), Path(id)) -> Result<impl IntoResponse, KoraError>` — same pattern as register_schema
- **Error pattern**: Add `SchemaNotFound` variant — maps to 404/40403. Keep it minimal.
- **Storage pattern**: Standalone async functions taking `&PgPool` as first arg
- **File placement**: `src/api/schemas.rs` per architecture doc (new file for schemas resource)
- **No `unwrap()`** outside `#[cfg(test)]`
- **`deny(missing_docs)` + `deny(clippy::pedantic)`** enforced

### Soft-Delete Behavior

Confluent Schema Registry returns schemas by global ID regardless of soft-delete status. The `GET /schemas/ids/{id}` endpoint does NOT filter by `deleted = false`. This is because deserializers in consumers need to resolve schema IDs from Kafka messages — those messages don't disappear when a schema is soft-deleted. The SQL query must NOT include `AND deleted = false`.

### File Structure

```
NEW:
├── src/api/schemas.rs          ← GET /schemas/ids/{id} handler
├── tests/api_get_schema.rs     ← Integration tests

MODIFIED:
├── src/api/mod.rs              ← add route + mod schemas
├── src/error.rs                ← add SchemaNotFound variant
├── src/storage/schemas.rs      ← add find_by_id
├── tests/error_responses.rs    ← add schema_not_found test
```

### Previous Story Intelligence

From story 1.2 completion:
- `KoraError` currently has only 2 variants: `InvalidSchema` (42201/422) and `BackendDataStore` (50001/500)
- Storage functions are standalone async fns taking `&PgPool` — follow same pattern
- Integration tests use `common::spawn_server()` + `common::pool()` for DB assertions
- `VALID_AVRO` constant used across tests: `r#"{"type":"record","name":"Test","fields":[{"name":"id","type":"int"}]}"#`
- Makefile auto-discovers test files: `api_*.rs` → integration, others → unit
- 25 tests currently (8 api_register + 9 schema_parsing + 3 config + 2 error + 2 db + 1 health)
- Body size configurable via `KORA_MAX_BODY_SIZE` (default 16MB)
- Content-Type middleware sets `application/vnd.schemaregistry.v1+json` globally

### Testing Strategy

- **Integration tests** (`tests/api_get_schema.rs`):
  - Register schema via POST, then GET by ID → verify response matches
  - GET non-existent ID → 404 + error code 40403
  - GET with non-numeric ID → 400 (axum rejects)
  - DB assertion: verify returned `schema` matches `schema_text` in DB
- **Unit test** (`tests/error_responses.rs`):
  - `schema_not_found` error serializes correctly

### References

- [Source: epics.md — Epic 1, Story 1.3]
- [Source: prd.md — FR2]
- [Source: architecture.md — schemas.rs: GET /schemas/ids/{id}]
- [Source: Confluent API v7.6 — GET /schemas/ids/{int: id}]
- [Source: 1-2-register-avro-schema.md — Completion Notes, Review Findings]

### Review Findings

- [x] [Review][Patch] AC #3 untested: no soft-delete retrieval test — FIXED: added `get_soft_deleted_schema_by_id_still_returns_200` test
- [x] [Review][Defer] Content-Type not set by error handler [src/error.rs] — deferred, pre-existing (middleware covers it)

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
