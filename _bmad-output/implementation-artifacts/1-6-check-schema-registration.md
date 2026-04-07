# Story 1.6: Check Schema Registration

Status: done

## Story

As a **developer**,
I want to check if a schema is already registered under a subject,
so that I can verify whether my schema exists without registering a new version.

## Acceptance Criteria

1. **Given** subject "orders-value" has a registered schema
   **When** I send `POST /subjects/orders-value` with `{"schema": "<matching_schema>"}`
   **Then** I receive HTTP 200 with `{"subject": "orders-value", "id": <id>, "version": <ver>, "schema": "<json>"}`

2. **Given** a schema not registered under the subject
   **When** I send `POST /subjects/orders-value` with `{"schema": "<unknown_schema>"}`
   **Then** I receive HTTP 404 with Confluent error code 40403

**FRs Covered:** FR5

## Tasks / Subtasks

- [x] Task 1: Storage layer — find schema by subject and fingerprint with full details (AC: #1)
  - [x] Add `schemas::find_by_subject_fingerprint(pool, subject, fingerprint) -> Result<Option<SchemaVersion>, sqlx::Error>` in `src/storage/schemas.rs`
  - [x] SQL: join schemas + subjects, match by subject name + fingerprint, filter `deleted = false`, return full SchemaVersion

- [x] Task 2: API handler — POST /subjects/{subject} (AC: #1, #2)
  - [x] Add `check_schema` handler in `src/api/subjects.rs`
  - [x] Accept same body as register: `{"schema": "...", "schemaType": "AVRO"}`
  - [x] Parse and validate schema (reuse `schema::parse`)
  - [x] Check subject exists → `SubjectNotFound` (40401) if missing
  - [x] Look up by fingerprint → return SchemaVersion if found, `SchemaNotFound` (40403) if not
  - [x] Register route in `src/api/mod.rs`: `POST /subjects/{subject}`

- [x] Task 3: Integration tests (AC: #1, #2)
  - [x] Create `tests/api_check_schema.rs`
  - [x] Test: register schema, then check → 200 with {subject, id, version, schema} (AC #1)
  - [x] Test: check unregistered schema → 404 + 40403 (AC #2)
  - [x] Test: check on non-existent subject → 404 + 40401

- [x] Task 4: Verify all tests pass
  - [x] `just lint` — zero warnings
  - [x] `just test` — all tests pass (43 existing + new)

## Dev Notes

### Confluent API Contract

```
POST /subjects/{subject}
Content-Type: application/vnd.schemaregistry.v1+json
Body: {"schema": "<schema_json>", "schemaType": "AVRO"}

Response (200 - found):
{"subject": "orders-value", "id": 1, "version": 1, "schema": "{...}", "schemaType": "AVRO"}

Response (404 - schema not registered under subject):
{"error_code": 40403, "message": "Schema not found"}

Response (404 - subject doesn't exist):
{"error_code": 40401, "message": "Subject not found"}
```

### Reuse from Previous Stories

- `SchemaVersion` struct (storage/schemas.rs) — already has the right fields + Serialize
- `schema::parse()` — parse and fingerprint the submitted schema
- `RegisterSchemaRequest` struct — same body format, reuse for deserialization
- `validate_subject()` — reuse for subject validation
- `subjects::exists()` — reuse for subject existence check
- Error variants `SubjectNotFound` (40401) and `SchemaNotFound` (40403) already exist

### Key Difference from Register (POST /subjects/{subject}/versions)

This endpoint **only checks** — it does NOT create a new version. It looks up by fingerprint and returns the existing entry or 404. Same body format, different path and behavior.

### References

- [Source: epics.md — Epic 1, Story 1.6]
- [Source: prd.md — FR5]
- [Source: architecture.md — Confluent API paths]
- [Source: 1-5-list-subjects-and-versions.md — Completion Notes]

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6 (1M context)

### Completion Notes List

- Added `schemas::find_by_subject_fingerprint()` — reuses `SchemaVersion` + `row_to_schema_version`
- Added `check_schema` handler — reuses `RegisterSchemaRequest`, `schema::parse`, `validate_subject`, `exists`
- No new structs or error variants needed — full reuse of existing code
- 3 new integration tests, 46 total, all passing
- Review: clean

### File List

**New:**
- `tests/api_check_schema.rs`

**Modified:**
- `src/api/mod.rs` — added route POST /subjects/{subject}
- `src/api/subjects.rs` — added `check_schema` handler
- `src/storage/schemas.rs` — added `find_by_subject_fingerprint()`
