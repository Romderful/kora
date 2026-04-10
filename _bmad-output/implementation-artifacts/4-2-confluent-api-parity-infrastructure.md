# Story 4.2: Confluent API Parity — Infrastructure & Core Params

Status: ready-for-dev

## Story

As a **developer using Confluent-compatible tooling** (confluent-kafka-python, confluent-kafka-go, Debezium, Kafka Connect, ksqlDB),
I want Kora's core infrastructure (error codes, pagination, root endpoint, config behavior) to match the Confluent Schema Registry,
so that client libraries work without modification.

## Acceptance Criteria

### AC1: Root Endpoint

**Given** a running Kora server
**When** I send `GET /`
**Then** I receive HTTP 200 with `{}` (empty JSON object — Confluent returns empty HashMap)

### AC2: Extended Error Codes

**Given** any API error
**When** the error response is returned
**Then** the following Confluent error codes are supported:

| Code | HTTP | Meaning |
|------|------|---------|
| 40401 | 404 | Subject not found |
| 40402 | 404 | Version not found |
| 40403 | 404 | Schema not found |
| 40404 | 404 | Subject was soft-deleted (re-registration guard) |
| 40405 | 404 | Subject NOT soft-deleted (hard-delete precondition) |
| 40406 | 404 | Schema version was soft-deleted |
| 40407 | 404 | Schema version NOT soft-deleted (hard-delete precondition) |
| 40408 | 404 | Subject compatibility level not configured |
| 40409 | 404 | Subject mode not configured |
| 40901 | 409 | Incompatible schema |
| 42201 | 422 | Invalid schema |
| 42202 | 422 | Invalid version |
| 42203 | 422 | Invalid compatibility level |
| 42204 | 422 | Invalid mode |
| 42205 | 422 | Operation not permitted |
| 42206 | 422 | Reference exists (cannot delete) |
| 50001 | 500 | Error in backend data store |
| 50002 | 500 | Operation timed out |
| 50003 | 500 | Error while forwarding request |

### AC3: Hard-Delete Error Code Fix

**Given** `DELETE /subjects/{subject}?permanent=true` and the subject is NOT soft-deleted
**Then** I receive HTTP 404 with error code **40405** (not 40401)

**Given** `DELETE /subjects/{subject}/versions/{version}?permanent=true` and the version is NOT soft-deleted
**Then** I receive HTTP 404 with error code **40407** (not 40402)

### AC4: Pagination

**Given** any list endpoint (`GET /subjects`, `GET /subjects/{subject}/versions`, `GET /schemas/ids/{id}/subjects`, `GET /schemas/ids/{id}/versions`)
**When** `offset` and `limit` query params are provided
**Then** results are paginated accordingly (`offset` default=0, `limit` default=-1 meaning unlimited)

### AC5: Config — Extended Params

**Given** `GET /config?defaultToGlobal=true`
**Then** returns the global config (no-op but param must be accepted without error)

**Given** `GET /config/{subject}?defaultToGlobal=true` and subject has NO per-subject config
**Then** returns the global compatibility level as fallback

**Given** `GET /config/{subject}` (no `defaultToGlobal`) and subject has NO per-subject config
**Then** returns HTTP 404 with error code **40408**

**Given** `DELETE /config`
**Then** the global config resets to BACKWARD (default)

**Given** `DELETE /config/{subject}`
**Then** returns `{"compatibilityLevel": "<previous_level>"}` — the **previous** value that was deleted, not the fallback

### AC6: schemaType Omission for AVRO

**Given** a registered AVRO schema
**When** I retrieve it via `GET /schemas/ids/{id}`
**Then** the `schemaType` field is **omitted** from the response (Confluent default: AVRO is implicit)

### AC7: Get Schema by ID — fetchMaxId + accept-and-ignore params

**Given** `GET /schemas/ids/{id}?fetchMaxId=true`
**Then** the response includes `"maxId": <highest_schema_id_in_registry>`

**Given** `GET /schemas/ids/{id}?subject=orders-value`
**Then** the param is accepted without error (accept and ignore for now)

**Given** `GET /schemas/ids/{id}?format=serialized` or `?referenceFormat=serialized`
**Then** the params are accepted without error (accept and ignore — no format transformation in MVP)

**Note:** `GET /schemas/ids/{id}` has NO `deleted` param in Confluent. Schemas are always retrievable by global ID regardless of subject soft-delete status.

### AC8: `format` and `referenceFormat` Params (accept-and-ignore)

**Given** any of these endpoints with a `format` or `referenceFormat` query param:
- `GET /schemas/ids/{id}` (`format`, `referenceFormat`)
- `GET /schemas/ids/{id}/schema` (`format`)
- `GET /subjects/{subject}/versions/{version}` (`format`, `referenceFormat`)
- `GET /subjects/{subject}/versions/{version}/schema` (`format`)
- `POST /subjects/{subject}` (`format`)
- `POST /subjects/{subject}/versions` (`format`)
**Then** the params are accepted without error (default `""`, no behavior change — accept and ignore in MVP)

### AC9: Version Validation

**Given** `GET /subjects/{subject}/versions/abc` (non-integer, non-"latest")
**Then** I receive HTTP 422 with error code **42202**

**Given** `GET /subjects/{subject}/versions/0` or any negative integer
**Then** I receive HTTP 422 with error code **42202**

**Note:** Confluent only accepts positive integers and the string `"latest"`. No negative indexing.

## Tasks / Subtasks

- [ ] Task 1: Root endpoint (AC: 1)
  - [ ] Add `GET /` route in `api/mod.rs` returning `{}`
  - [ ] Add `POST /` route returning `{}` (Confluent has both)
  - [ ] Tests: GET and POST root return empty JSON

- [ ] Task 2: Extended error codes (AC: 2)
  - [ ] Add new `KoraError` variants: `SubjectSoftDeleted` (40404), `SubjectNotSoftDeleted` (40405), `SchemaVersionSoftDeleted` (40406), `SchemaVersionNotSoftDeleted` (40407), `SubjectCompatibilityNotConfigured` (40408), `SubjectModeNotConfigured` (40409), `InvalidVersion` (42202), `InvalidMode` (42204), `OperationNotPermitted` (42205), `OperationTimeout` (50002), `ForwardingError` (50003)
  - [ ] Implement `error_code()` and `status_code()` for each
  - [ ] Tests: verify each new error code returns correct HTTP status and Confluent format

- [ ] Task 3: Hard-delete error code fix (AC: 3)
  - [ ] In `delete_subject`: when `permanent=true`, check if subject `deleted=true` first; return 40405 if not soft-deleted
  - [ ] In `delete_version`: same logic, return 40407 if version not soft-deleted
  - [ ] Tests: hard-delete non-soft-deleted subject → 40405, hard-delete non-soft-deleted version → 40407

- [ ] Task 4: Pagination infrastructure (AC: 4)
  - [ ] Create shared `PaginationParams` struct: `offset: i64` (default 0), `limit: i64` (default -1) with `#[serde(default)]`
  - [ ] Extend storage functions to accept offset/limit: `list_subjects`, `list_schema_versions`, `find_subjects_by_schema_id`, `find_versions_by_schema_id`
  - [ ] SQL: append `OFFSET $N` always; append `LIMIT $M` only when limit >= 0
  - [ ] Update API handlers to extract and pass pagination params
  - [ ] Tests: with offset/limit, default (no params), limit=-1 returns all

- [ ] Task 5: Config extended params (AC: 5)
  - [ ] Add `DefaultToGlobalParam { default_to_global: bool }` with `#[serde(rename = "defaultToGlobal")]`
  - [ ] `GET /config`: accept param (no behavior change — it IS the global)
  - [ ] `GET /config/{subject}`: without `defaultToGlobal=true`, return 40408 if no subject config exists
  - [ ] `GET /config/{subject}?defaultToGlobal=true`: fallback to global (current behavior)
  - [ ] Add `DELETE /config` route: reset global to BACKWARD; accept `subject` query param (accept-and-ignore, Confluent OSS has it)
  - [ ] Fix `DELETE /config/{subject}`: return the **deleted** level with key `compatibilityLevel`, not the fallback
  - [ ] Tests: defaultToGlobal=true fallback, no param → 40408, global DELETE resets, subject DELETE returns previous level

- [ ] Task 6: schemaType omission + fetchMaxId (AC: 6, 7)
  - [ ] Modify `get_schema_by_id` response: omit `schemaType` field when value is `"AVRO"`
  - [ ] Add `GetSchemaByIdParams { fetch_max_id: bool, subject: Option<String>, format: String, reference_format: String }` with serde renames
  - [ ] `fetchMaxId=true`: add `SELECT MAX(id) FROM schemas` query, add `maxId` to response
  - [ ] `subject`, `format`, `referenceFormat`: accept params, no behavior change (accept-and-ignore)
  - [ ] **No `deleted` param** — schemas are always retrievable by global ID regardless of subject soft-delete
  - [ ] Tests: AVRO response omits schemaType, non-AVRO includes it, fetchMaxId returns correct max, format param accepted

- [ ] Task 7: `format` / `referenceFormat` accept-and-ignore (AC: 8)
  - [ ] Add `format: String` (default `""`) query param to: `get_schema_by_version`, `get_schema_only` (story 4.4), `check_schema`, `register_schema`
  - [ ] Add `referenceFormat: String` (default `""`) to: `get_schema_by_version`
  - [ ] All accept-and-ignore in MVP — no format transformation behavior
  - [ ] Tests: endpoints accept format param without error

- [ ] Task 8: Version validation fix (AC: 9)
  - [ ] Update version parsing: accept only positive integers or `"latest"`
  - [ ] Return error code 42202 for: non-integer strings (except "latest"), zero, negative numbers
  - [ ] Tests: "latest" works, positive int works, 0 → 42202, -1 → 42202, "abc" → 42202

## Dev Notes

### Architecture Compliance

- **New routes**: `GET /` + `POST /`, `DELETE /config` — everything else is param additions
- **Pattern**: combined `#[derive(Deserialize)]` structs with `#[serde(default)]` and `#[serde(rename = "camelCase")]`
- **Storage layer**: extend existing function signatures — do NOT create new functions
- **SQL pagination**: `OFFSET $N LIMIT $M` where LIMIT omitted when -1

### Existing Infrastructure to Reuse

- `DeletedParam` in `subjects.rs` — keep as-is for now, extend in story 4.3
- `PermanentParam` in `subjects.rs` — no changes
- `canonical_form` and `fingerprint` columns — already populated
- `mode` column in `config` table — stored but unused until story 5.1
- `KoraError` enum — add new variants following existing pattern
- `COMPATIBILITY_LEVELS` constant — already defined

### Response Format Rules (Confluent Compatibility)

- `GET /` returns `{}` (empty JSON object)
- `GET /config` returns `{"compatibilityLevel": "..."}` (with `Level` suffix)
- `PUT /config` returns `{"compatibility": "..."}` (without suffix — **different on purpose**)
- `DELETE /config/{subject}` returns `{"compatibilityLevel": "..."}` — the **previous** value
- `GET /schemas/ids/{id}` omits `schemaType` when AVRO; NO `deleted` param (schemas always retrievable by ID)

### Version Parsing (CRITICAL — Confluent VersionId behavior)

Confluent's `VersionId` class:
- `"latest"` → internally stored as -1
- Positive integers → accepted as-is
- **Everything else is rejected**: 0, negative numbers, non-numeric strings → `InvalidVersionException`
- No negative indexing (-2, -3, etc.) — this is a common misconception

### Files to Modify

| File | Changes |
|------|---------|
| `src/api/mod.rs` | Add `GET /`, `POST /`, `DELETE /config` routes |
| `src/api/schemas.rs` | fetchMaxId/subject/format/referenceFormat params, omit schemaType for AVRO |
| `src/api/subjects.rs` | Add format param to register_schema, check_schema, get_schema_by_version |
| `src/api/compatibility.rs` | defaultToGlobal param, fix DELETE response, add global DELETE |
| `src/error.rs` | Add 40404-40409, 42202, 42204, 42205, 50002, 50003 |
| `src/storage/subjects.rs` | Add pagination params to list_subjects |
| `src/storage/schemas.rs` | Add pagination params to cross-reference queries, version validation |
| `src/storage/compatibility.rs` | defaultToGlobal logic, global delete, fix subject delete response |
| `tests/api_confluent_parity_infra.rs` | New test file for infrastructure ACs |
| `tests/common/api.rs` | Add helper functions |

### References

- [Source: Confluent Schema Registry Java — RootResource.java] — GET/POST / return empty HashMap
- [Source: Confluent Schema Registry Java — VersionId.java] — only "latest" and positive ints accepted
- [Source: Confluent Schema Registry Java — Errors.java] — complete error code list (40401-50003)
- [Source: _bmad-output/planning-artifacts/epics.md] — Stories 1.1–4.1 acceptance criteria
- [Source: _bmad-output/planning-artifacts/architecture.md] — API patterns, naming conventions

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### Change Log

### File List
