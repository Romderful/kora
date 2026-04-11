# Story 4.3: Confluent API Parity — List & Lookup Params

Status: ready-for-dev

**Depends on:** Story 4.2 (pagination infrastructure, error codes)

## Story

As a **developer using Confluent-compatible tooling**,
I want all list and lookup endpoints to support the full set of Confluent query parameters (subjectPrefix, deletedOnly, deletedAsNegative, normalize, deleted on cross-refs and check),
so that every Confluent client library query works identically against Kora.

## Acceptance Criteria

### AC1: List Subjects — Full Param Support

**Given** `GET /subjects` with no `subjectPrefix` param
**Then** all subjects are returned (default `subjectPrefix` = `:*:` = match all)

**Given** `GET /subjects?subjectPrefix=orders`
**Then** only subjects whose name starts with "orders" are returned

**Given** `GET /subjects?deletedOnly=true`
**Then** ONLY soft-deleted subjects are returned (not active ones)

**Given** `GET /subjects?deleted=true&deletedOnly=true`
**Then** `deletedOnly` takes precedence — only soft-deleted subjects returned

### AC2: List Versions — Full Param Support

**Given** `GET /subjects/{subject}/versions?deletedOnly=true`
**Then** only soft-deleted versions are returned

**Given** `GET /subjects/{subject}/versions?deletedAsNegative=true`
**Then** soft-deleted versions appear as negative numbers (e.g., version 2 soft-deleted → -2 in the list)
**And** active versions appear as positive numbers

### AC3: Register Schema — `normalize` Param

**Given** `POST /subjects/{subject}/versions?normalize=true`
**When** I register a schema
**Then** the schema is normalized (canonical_form) before fingerprint comparison and storage deduplication

**Given** two schemas that differ only in whitespace/ordering
**When** both registered with `normalize=true`
**Then** the second registration returns the same ID (deduplicated via canonical form)

### AC4: Check Schema — Extended Params

**Given** `POST /subjects/{subject}?normalize=true`
**When** checking if a schema is registered
**Then** the lookup uses the normalized (canonical) form for comparison

**Given** `POST /subjects/{subject}?deleted=true`
**Then** soft-deleted schema matches are included in the lookup result

### AC5: Cross-References — `deleted` + `subject` Params

**Given** `GET /schemas/ids/{id}/subjects?deleted=true`
**Then** soft-deleted subjects are included in the result

**Given** `GET /schemas/ids/{id}/versions?deleted=true`
**Then** soft-deleted versions are included in the result

**Given** `GET /schemas/ids/{id}/subjects?subject=orders-value`
**Then** results are filtered to only the "orders-value" subject

**Given** `GET /schemas/ids/{id}/versions?subject=orders-value`
**Then** results are filtered to versions under "orders-value" only

### AC6: Get Schema by Version — `deleted` Param

**Given** a soft-deleted version 2 of subject "orders-value"
**When** I send `GET /subjects/orders-value/versions/2?deleted=true`
**Then** the soft-deleted version is returned

**Given** a soft-deleted version 2 and no `deleted` param
**When** I send `GET /subjects/orders-value/versions/2`
**Then** I receive HTTP 404 with error code 40402

### AC7: Referencedby — `deleted` Param

**Given** `GET /subjects/{subject}/versions/{version}/referencedby?deleted=true`
**Then** soft-deleted referencing schemas are included

**Note:** referencedby endpoint is from story 4.5 (not yet implemented). This AC applies once 4.5 is done.

## Tasks / Subtasks

- [ ] Task 1: `subjectPrefix` + `deletedOnly` on list subjects (AC: 1)
  - [ ] Extend `ListParams` into `ListSubjectsParams { deleted: bool, deleted_only: bool, lookup_deleted_subject: bool, subject_prefix: String, offset: i64, limit: i64 }` with serde renames (`deletedOnly`, `lookupDeletedSubject`, `subjectPrefix`)
  - [ ] `lookupDeletedSubject`: accept-and-ignore (Confluent OSS has it, overlaps with `deleted`/`deletedOnly`)
  - [ ] Default `subject_prefix` to `":*:"` via custom serde default
  - [ ] Extend `storage::list_subjects` to accept prefix + deleted_only
  - [ ] SQL: add `WHERE name LIKE $prefix%` (skip when prefix is `:*:`), add `WHERE deleted = true` for deletedOnly
  - [ ] Tests: prefix filtering, deletedOnly, default behavior

- [ ] Task 2: `deletedOnly` + `deletedAsNegative` on list versions (AC: 2)
  - [ ] Extend `ListParams` into `ListVersionsParams { deleted: bool, deleted_only: bool, deleted_as_negative: bool, offset: i64, limit: i64 }` with serde renames
  - [ ] Extend `storage::list_schema_versions` to support `deleted_only` and `deleted_as_negative`
  - [ ] For `deletedAsNegative`: SQL returns `-version` for deleted rows, `version` for active rows, ordered by `abs(version)`
  - [ ] Tests: deletedOnly, deletedAsNegative with mix of active/deleted versions

- [ ] Task 3: `normalize` on register + check (AC: 3, 4)
  - [ ] Add `normalize: bool` query param to `register_schema` and `check_schema` handlers
  - [ ] When `normalize=true` on register: compare fingerprints using canonical_form (already stored) instead of raw schema text
  - [ ] When `normalize=true` on check: same — use canonical form for lookup
  - [ ] Tests: register two formatting-different schemas with normalize=true → same ID; check with normalize=true finds match

- [ ] Task 4: `deleted` param on check_schema (AC: 4)
  - [ ] Add `deleted: bool` to check_schema handler query params
  - [ ] When `deleted=true`: include soft-deleted schemas in lookup
  - [ ] Modify `storage::find_schema_by_subject_id_and_fingerprint` to accept `include_deleted`
  - [ ] Tests: check against soft-deleted schema with deleted=true returns match

- [ ] Task 5: `deleted` + `subject` params on cross-references (AC: 5)
  - [ ] Add `deleted: bool` and `subject: Option<String>` to `get_subjects_by_schema_id` and `get_versions_by_schema_id` handlers
  - [ ] `subject` param filters results to a specific subject name (Confluent uses this to scope cross-reference lookups)
  - [ ] Modify `storage::find_subjects_by_schema_id` and `find_versions_by_schema_id` to accept `include_deleted` and optional `subject` filter
  - [ ] Tests: cross-refs include soft-deleted when deleted=true; subject filter narrows results

- [ ] Task 6: `deleted` param on get-by-version (AC: 6)
  - [ ] Add `deleted: bool` query param to `get_schema_by_version` handler
  - [ ] Modify `storage::find_schema_by_subject_version` to accept `include_deleted`
  - [ ] Tests: get soft-deleted version with deleted=true → 200; without → 404

## Dev Notes

### Architecture Compliance

- **No new routes** — all changes are query param additions to existing handlers
- **Extend existing `ListParams`** into richer param structs per handler
- **Storage layer**: add boolean/string params to existing functions — do NOT create new functions
- **Pagination** from story 4.2 is already wired — compose with new param structs via `#[serde(flatten)]` or explicit fields

### Query Param Struct Pattern

```rust
#[derive(Debug, Deserialize)]
pub struct ListSubjectsParams {
    #[serde(default)]
    pub deleted: bool,
    #[serde(default, rename = "deletedOnly")]
    pub deleted_only: bool,
    #[serde(default = "default_subject_prefix", rename = "subjectPrefix")]
    pub subject_prefix: String,
    #[serde(default)]
    pub offset: i64,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_subject_prefix() -> String { ":*:".to_string() }
fn default_limit() -> i64 { -1 }
```

### `normalize` Implementation

The `canonical_form` column is already populated for every schema at registration time. When `normalize=true`:
- **Register**: compute fingerprint from `canonical_form` instead of raw `schema_text` for dedup check
- **Check/Lookup**: same — match against canonical fingerprint
- Current code in `subjects.rs` uses `schema::parse()` which returns `ParsedSchema { canonical_form, fingerprint }`. The fingerprint is already based on canonical form. So `normalize=true` may already be the default behavior — **verify before implementing**. If fingerprint is already canonical-based, `normalize` is a no-op (accept the param, no behavior change).

### `deletedAsNegative` SQL Pattern

```sql
SELECT CASE WHEN s.deleted THEN -s.version ELSE s.version END as version
FROM schemas s JOIN subjects sub ON s.subject_id = sub.id
WHERE sub.name = $1
ORDER BY abs(s.version)
OFFSET $2
-- LIMIT $3 (omit when -1)
```

### Files to Modify

| File | Changes |
|------|---------|
| `src/api/subjects.rs` | Extend ListParams into richer structs, add normalize/deleted params |
| `src/api/schemas.rs` | Add deleted param to cross-reference handlers |
| `src/storage/subjects.rs` | Add subjectPrefix, deletedOnly to list_subjects |
| `src/storage/schemas.rs` | Add deletedOnly, deletedAsNegative to list_versions; add include_deleted to cross-ref + version queries |
| `tests/api_confluent_parity_params.rs` | New test file for list/lookup param ACs |
| `tests/common/api.rs` | Extend helpers with new param support |

### References

- [Source: Confluent Java — SubjectsResource.java] — `subjectPrefix` default `:*:`, `deletedOnly` param
- [Source: Confluent Java — SubjectVersionsResource.java] — `deletedAsNegative`, `deletedOnly` params
- [Source: _bmad-output/planning-artifacts/epics.md] — Stories 1.5, 1.6, 2.3 acceptance criteria

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### Change Log

### File List
