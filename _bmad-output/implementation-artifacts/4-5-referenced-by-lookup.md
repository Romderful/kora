# Story 4.5: Referenced-By Lookup

Status: done

**Depends on:** Story 4.2 (pagination, deleted param)

## Story

As a **developer**,
I want to find which schemas reference a given schema version,
so that I can understand downstream dependencies before making changes.

## Acceptance Criteria

### AC1: Basic Referenced-By Lookup

**Given** schema "orders-value" v1 references "users-value" v1
**When** I send `GET /subjects/users-value/versions/1/referencedby`
**Then** I receive HTTP 200 with `[<schema_id_of_orders_v1>]` (array of content IDs)

### AC2: No Dependents

**Given** a schema version with no dependents
**When** I send `GET /subjects/orders-value/versions/1/referencedby`
**Then** I receive HTTP 200 with `[]`

### AC3: Pagination

**Given** pagination on referencedby
**When** I send `GET /subjects/users-value/versions/1/referencedby?offset=0&limit=10`
**Then** results are paginated (limit=-1 means unlimited, default)

### AC4: Soft-Deleted Referencing Schemas

**Given** `deleted=true`
**When** I send `GET /subjects/users-value/versions/1/referencedby?deleted=true`
**Then** soft-deleted referencing schema versions are included

**Given** default (no `deleted` param)
**Then** only content IDs with at least one active (non-deleted) schema version are returned

### AC5: Error Codes

**Given** a non-existent subject
**When** I send `GET /subjects/unknown/versions/1/referencedby`
**Then** I receive HTTP 404 with Confluent error code 40401

**Given** an existing subject but non-existent version
**When** I send `GET /subjects/users-value/versions/99/referencedby`
**Then** I receive HTTP 404 with Confluent error code 40402

**Given** an invalid version (0, negative, non-numeric)
**When** I send `GET /subjects/users-value/versions/0/referencedby`
**Then** I receive HTTP 422 with error code 42202

**FRs:** FR36

## Tasks / Subtasks

- [x] Task 1: Storage function — `find_referencing_schema_ids` (AC: 1, 2, 3, 4)
  - [x] 1.1: Add `find_referencing_schema_ids` to `src/storage/references.rs`
    - Query: `SELECT DISTINCT sr.content_id FROM schema_references sr JOIN schema_versions sv ON sr.content_id = sv.content_id WHERE sr.subject = $1 AND sr.version = $2 AND (sv.deleted = false OR $include_deleted) ORDER BY sr.content_id OFFSET $n [LIMIT $m]`
    - Params: `pool, target_subject, target_version, include_deleted, offset, limit`
    - Returns: `Vec<i64>` (schema content IDs)
    - Match on `limit >= 0` for WITH/WITHOUT LIMIT variants (established pattern)
  - [x] 1.2: Tests for the storage function via integration tests (covered by handler tests)

- [x] Task 2: Handler + query params + route (AC: 1, 2, 3, 4, 5)
  - [x] 2.1: Create `ReferencedByParams` query param struct in `src/api/subjects.rs`
    - `deleted: bool` (default false)
    - `offset: i64` (default 0)
    - `limit: i64` (default -1 via `default_limit`)
  - [x] 2.2: Create `get_referencedby` handler in `src/api/subjects.rs`
    - `validate_subject(&subject)`
    - Parse version (reuse `parse_version`, support only positive integers — NOT "latest")
    - Check subject exists → 40401
    - Check version exists → 40402
    - Call `references::find_referencing_schema_ids`
    - Return `Json(ids)` — array of i64
  - [x] 2.3: Register route in `src/api/mod.rs`
    - `.route("/subjects/{subject}/versions/{version}/referencedby", get(subjects::get_referencedby))`
  - [x] 2.4: Tests
    - Schema with references → returns referencing schema IDs
    - No references → returns `[]`
    - Multiple referencing schemas → returns all IDs
    - Soft-deleted referencing version excluded by default
    - Soft-deleted referencing version included with `deleted=true`
    - Pagination (offset + limit)
    - Non-existent subject → 404 / 40401
    - Non-existent version → 404 / 40402
    - Invalid version (0, negative) → 422 / 42202

## Dev Notes

### Architecture Compliance

- **Storage function** in `src/storage/references.rs` — this is a reference-related query, not a schema query
- **Handler** in `src/api/subjects.rs` — the route is under `/subjects/{subject}/versions/{version}/...`
- **Reuse** `validate_subject`, `parse_version`, `default_limit`, `GetVersionParams` pattern
- **Return type**: `Vec<i64>` — Confluent returns an array of schema IDs (content IDs), not subject-version pairs

### "latest" Not Supported

Confluent's `referencedby` endpoint does NOT support `"latest"` as a version. Only positive integers. The handler should use `parse_version` directly (no "latest" branch).

### SQL Query

The reverse lookup joins `schema_references` → `schema_versions` to find content IDs that reference the target:

```sql
SELECT DISTINCT sr.content_id
FROM schema_references sr
JOIN schema_versions sv ON sr.content_id = sv.content_id
WHERE sr.subject = $1 AND sr.version = $2
  AND (sv.deleted = false OR $3)
ORDER BY sr.content_id
OFFSET $4
-- LIMIT $5 (only when >= 0)
```

This is consistent with the existing `is_version_referenced` function which uses the same JOIN pattern but only checks existence.

### Existence Checks

The handler must validate subject and version exist BEFORE querying references:
1. `subjects::subject_exists(pool, subject, false)` — 40401 if not found
2. Check version exists: query `schema_versions` for the subject/version combo. If not found → 40402.

For the version check, reuse `schemas::find_schema_by_subject_version(pool, subject, version, false)` and check if it returns `None`. This is the lazy pattern — on the happy path where references exist, the version necessarily exists.

Actually, simpler: call `find_referencing_schema_ids` first. If the result is non-empty, the version exists. If empty, THEN check subject/version existence to distinguish between "no references" (200 + []) and "not found" (40401/40402).

### Files to Modify

| File | Changes |
|------|---------|
| `src/storage/references.rs` | Add `find_referencing_schema_ids` function |
| `src/api/subjects.rs` | Add `ReferencedByParams` struct, `get_referencedby` handler |
| `src/api/mod.rs` | Register `/subjects/{subject}/versions/{version}/referencedby` route |

### Testing Standards

- Integration tests against the HTTP API (spawn_server pattern)
- Use schema references setup: register a base schema, then register a schema that references it
- Use `unique_avro_schema()` for content isolation
- UUID-based subject names
- Verify HTTP status codes AND Confluent error codes

### Previous Story Intelligence

From Story 4.4:
- Pagination pattern: match on `limit >= 0` for WITH/WITHOUT LIMIT SQL variants
- `default_limit()` returns -1 (unlimited), defined in `subjects.rs` as `pub(crate) const fn`
- Error handling: check existence lazily — only when main query returns empty/None
- `validate_subject` and `parse_version` are private helpers in `subjects.rs`, reusable within the file

From Story 4.3 (AC7):
- The `deleted` param on referencedby was deferred to this story. Now implementing it.

From Story 2.4:
- `schema_references` table structure established. `content_id` is the FK to `schema_contents`.
- `is_version_referenced` function exists — same JOIN pattern, but checks existence only.
- Reference tests in `tests/api_schema_references.rs` show how to set up referencing schemas.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 4.5] — AC definitions
- [Source: _bmad-output/planning-artifacts/prd.md#FR36] — Functional requirement
- [Source: src/storage/references.rs] — `is_version_referenced` JOIN pattern, `find_references_by_schema_id`
- [Source: src/api/subjects.rs] — `get_schema_by_version` error handling pattern, `parse_version`, `validate_subject`
- [Source: src/api/schemas.rs] — `CrossRefParams` pagination pattern
- [Source: tests/api_schema_references.rs] — Reference test setup patterns

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6 (1M context)

### Debug Log References

### Completion Notes List

- **Task 1**: Added `find_referencing_schema_ids` to `storage/references.rs`. Reverse lookup: joins `schema_references` → `schema_versions` on `content_id`, filters by target subject/version, respects soft-delete via `include_deleted`, returns `Vec<i64>` content IDs. Two SQL variants for WITH/WITHOUT LIMIT.
- **Task 2**: Added `ReferencedByParams` (deleted, offset, limit) and `get_referencedby` handler in `subjects.rs`. Lazy existence check: only queries subject/version when result is empty (avoids extra queries on happy path). Registered route. 9 integration tests covering all ACs.

### Change Log

- 2026-04-13: Implemented story 4.5. Added `GET /subjects/{subject}/versions/{version}/referencedby` endpoint returning `Vec<i64>` content IDs. 3 files modified, 9 new tests in existing module, 181 total passing, clippy pedantic clean.
- 2026-04-13: Post-review — renamed `get_referencedby` → `get_referencing_ids_by_version` (follows `get_<what>_by_<key>` convention). Reordered handler after `get_schema_text_by_version` to match route order in mod.rs. Moved `ReferencedByParams` after `PermanentParams` for consistent struct ordering. Inlined test setup (removed `setup_reference` helper) for consistency with existing tests in module.

### File List

- src/storage/references.rs (modified) — added `find_referencing_schema_ids` function
- src/api/subjects.rs (modified) — added `ReferencedByParams` struct, `get_referencing_ids_by_version` handler
- src/api/mod.rs (modified) — registered `/subjects/{subject}/versions/{version}/referencedby` route
- tests/api_schema_references.rs (modified) — +9 referencedby integration tests
