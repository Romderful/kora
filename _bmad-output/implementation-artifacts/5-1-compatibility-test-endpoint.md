# Story 5.1: Compatibility Test Endpoint

Status: done

**Depends on:** Story 4.1 (compatibility config CRUD), Story 4.3 (normalize param)

## Story

As a **developer**,
I want to test if a new schema is compatible with existing versions,
so that I can validate schema changes before registering them.

## Acceptance Criteria

### AC1: Test Against Specific Version — Compatible

**Given** subject "orders-value" with version 1 and compatibility mode BACKWARD
**When** I send `POST /compatibility/subjects/orders-value/versions/latest` with a backward-compatible schema
**Then** I receive HTTP 200 with `{"is_compatible": true}`

### AC2: Test Against Specific Version — Incompatible

**Given** subject "orders-value" with compatibility mode BACKWARD
**When** I send `POST /compatibility/subjects/orders-value/versions/latest` with an incompatible schema
**Then** I receive HTTP 200 with `{"is_compatible": false}`

### AC3: Verbose Mode

**Given** subject "orders-value" with compatibility mode BACKWARD and `verbose=true`
**When** I send `POST /compatibility/subjects/orders-value/versions/latest?verbose=true` with an incompatible schema
**Then** I receive HTTP 200 with `{"is_compatible": false, "messages": ["...descriptive incompatibility messages..."]}`

### AC4: Test Against All Versions

**Given** subject "orders-value" with versions 1, 2, 3
**When** I send `POST /compatibility/subjects/orders-value/versions` with a new schema
**Then** compatibility is tested against ALL versions (1, 2, 3)

### AC5: Normalize

**Given** `normalize=true` on compatibility check
**When** I send `POST /compatibility/subjects/orders-value/versions/latest?normalize=true`
**Then** the schema is normalized before compatibility testing

### AC6: Error Codes

**Given** a non-existent subject
**When** I send `POST /compatibility/subjects/unknown/versions/latest`
**Then** I receive HTTP 404 with Confluent error code 40401

**Given** a non-existent version
**When** I send `POST /compatibility/subjects/orders-value/versions/99`
**Then** I receive HTTP 404 with error code 40402

**Given** an invalid schema in the request body
**When** I send `POST /compatibility/subjects/orders-value/versions/latest` with unparseable schema
**Then** I receive HTTP 422 with error code 42201

**FRs:** FR20, FR37, FR38

## Tasks / Subtasks

- [x] Task 1: Schema compatibility check functions (AC: 1, 2, 3)
  - [x] 1.1: Add `check_compatibility` to `src/schema/avro.rs`
    - Use `apache_avro::schema_compatibility::SchemaCompatibility`
    - `can_read(reader, writer)` checks if reader schema can read data written by writer
    - BACKWARD: `can_read(new, old)` — new can read old data
    - FORWARD: `can_read(old, new)` — old can read new data
    - FULL: both directions
    - Return `CompatibilityResult { is_compatible: bool, messages: Vec<String> }`
    - Extract incompatibility messages from `CompatibilityError` for verbose mode
  - [x] 1.2: Add `check_compatibility` to `src/schema/json_schema.rs`
    - No standard compat library — implement basic structural rules:
      - BACKWARD: new schema must accept everything old schema accepts (superset)
      - FORWARD: old schema must accept everything new schema accepts
      - For MVP: use schema equality check (same canonical form = compatible, different = check deeper)
    - Return `CompatibilityResult`
  - [x] 1.3: Add `check_compatibility` to `src/schema/protobuf.rs`
    - Wire-compatibility rules: field number reuse, type changes, required field additions
    - For MVP: basic field number and type checking
    - Return `CompatibilityResult`
  - [x] 1.4: Add `check_compatibility` dispatch in `src/schema/mod.rs`
    - `pub fn check_compatibility(format: SchemaFormat, new_schema: &str, existing_schema: &str) -> Result<CompatibilityResult, KoraError>`
    - Parse both schemas, dispatch to format-specific checker
    - Define `CompatibilityResult` struct

- [x] Task 2: Effective compatibility level resolution (AC: 1, 2, 4)
  - [x] 2.1: Add `get_effective_compatibility` to `src/storage/compatibility.rs`
    - Subject-level → global fallback (same pattern as `get_effective_normalize`)
    - Returns the effective compatibility level as a String

- [x] Task 3: Compatibility test handlers + routes (AC: 1, 2, 3, 4, 5, 6)
  - [x] 3.1: Create `CompatibilityTestRequest` struct in `src/api/compatibility.rs`
    - `schema: String` — the new schema text
    - `schemaType: Option<String>` — format (default AVRO)
    - `references: Option<Vec<SchemaReference>>` — schema references
  - [x] 3.2: Create `CompatibilityTestParams` query param struct
    - `verbose: bool` (default false)
    - `normalize: bool` (default false)
  - [x] 3.3: Create `test_compatibility_by_version` handler
    - `POST /compatibility/subjects/{subject}/versions/{version}`
    - Parse version (support "latest" and positive integers)
    - Resolve effective compatibility level for subject
    - Parse new schema, fetch existing schema by version
    - Run `check_compatibility` based on mode direction
    - Return `{"is_compatible": bool}` (+ `"messages"` when verbose=true)
    - Errors: 40401 subject not found, 40402 version not found, 42201 invalid schema
  - [x] 3.4: Create `test_compatibility_against_all_versions` handler
    - `POST /compatibility/subjects/{subject}/versions`
    - Fetch ALL active versions for the subject
    - Run compatibility check against each version
    - Aggregate: `is_compatible = true` only if ALL checks pass
    - Collect all messages for verbose mode
    - Return same response shape
  - [x] 3.5: Register routes in `src/api/mod.rs`
    - `.route("/compatibility/subjects/{subject}/versions/{version}", post(compatibility::test_compatibility_by_version))`
    - `.route("/compatibility/subjects/{subject}/versions", post(compatibility::test_compatibility_against_all_versions))`
  - [x] 3.6: Normalize support
    - When `normalize=true`: use canonical form for compatibility testing
    - Resolve effective normalize from config (params.normalize OR subject/global config)

- [x] Task 4: Tests
  - [x] 4.1: Avro BACKWARD — compatible (add optional field with default)
  - [x] 4.2: Avro BACKWARD — incompatible (remove required field)
  - [x] 4.3: Avro FORWARD — compatible (remove optional field)
  - [x] 4.4: Avro FORWARD — incompatible (add required field without default)
  - [x] 4.5: Avro FULL — must be both backward and forward compatible
  - [x] 4.6: NONE mode — always compatible
  - [x] 4.7: verbose=true returns messages for incompatible schemas
  - [x] 4.8: Test against all versions endpoint
  - [x] 4.9: Test against "latest" version
  - [x] 4.10: Non-existent subject → 40401
  - [x] 4.11: Non-existent version → 40402
  - [x] 4.12: Invalid schema → 42201

## Dev Notes

### Architecture Compliance

- **Schema compatibility functions** in `src/schema/{avro,json_schema,protobuf}.rs` — each format implements its own check
- **Dispatch** in `src/schema/mod.rs` — `check_compatibility` routes to format-specific function
- **Handlers** in `src/api/compatibility.rs` — next to existing config handlers
- **Storage** — add `get_effective_compatibility` to `src/storage/compatibility.rs`
- **No new modules** — all changes in existing files

### Compatibility Mode Direction

The compatibility mode determines the CHECK DIRECTION, not which versions to check:

| Mode | Direction | Meaning |
|------|-----------|---------|
| BACKWARD | `can_read(new, old)` | New schema can read old data |
| FORWARD | `can_read(old, new)` | Old schema can read new data |
| FULL | Both directions | Both must hold |
| NONE | Skip | Always compatible |

TRANSITIVE modes (BACKWARD_TRANSITIVE, FORWARD_TRANSITIVE, FULL_TRANSITIVE) apply the same direction but are relevant for enforcement (story 5.2), not the test endpoint. The test endpoints:
- `/versions/{version}` tests against the specified version
- `/versions` tests against ALL versions

### Avro Compatibility via apache-avro

The `apache-avro` crate (v0.21.0) provides:
```rust
use apache_avro::schema::Schema;
use apache_avro::schema_compatibility::SchemaCompatibility;

// Check if reader can read data written by writer
let result = SchemaCompatibility::can_read(&reader_schema, &writer_schema);
// Returns: Result<(), CompatibilityError>
// CompatibilityError contains incompatibility details for verbose mode
```

BACKWARD check: `can_read(new_schema, existing_schema)` — new is reader, old is writer
FORWARD check: `can_read(existing_schema, new_schema)` — old is reader, new is writer

### JSON Schema Compatibility — Full Confluent Diff Engine

Complete structural diff engine matching Confluent's `COMPATIBLE_CHANGES_STRICT` set (57 types).
Implemented in `src/schema/json_schema/diff.rs`:
- **111 DiffType variants** covering all Confluent `Difference.Type` values (except `RESERVED_PROPERTY_*` which are Confluent Cloud proprietary)
- Type comparison with `integer → number` promotion
- String constraints (maxLength, minLength, pattern)
- Number constraints (min, max, exclusive, multipleOf via **divisibility** check)
- Object properties (3-tier model: open / partially open / closed, with regex `patternProperties` matching)
- Dependencies (array + schema forms)
- Array items (single schema + tuple via `items` and `prefixItems`)
- Enum (extended/narrowed/changed)
- Combined schemas (allOf/anyOf/oneOf with **maximum bipartite matching** via augmenting paths)
- `not` schema (reversed comparison per Confluent semantics)
- `const` changes (mapped to `ENUM_ARRAY_CHANGED`)
- `$ref` resolution (single-document JSON Pointer)
- `FalseSchema`/`EmptySchema` special handling
- Singleton oneOf/anyOf unwrapping

### Protobuf Compatibility — Full Confluent Diff Engine

Complete structural diff engine matching Confluent's `COMPATIBLE_CHANGES` set (14 types).
Implemented in `src/schema/protobuf/diff.rs`:
- **25 DiffType variants** matching all Confluent `Difference.Type` values
- Field comparison by number (not name), type classification (scalar vs message/enum)
- Label changes: string/bytes = compatible, numeric = incompatible
- Required field additions/removals
- Oneof field move detection (single vs multiple)
- Message add/remove/move, nested recursion
- Enum add/remove, const add/change/remove

### Confluent Response Format

```json
// Compatible
{"is_compatible": true}

// Incompatible (non-verbose)
{"is_compatible": false}

// Incompatible (verbose)
{"is_compatible": false, "messages": ["Incompatibility: reader missing field 'name'"]}
```

Note: Confluent uses `is_compatible` (underscore), not `isCompatible` (camelCase).

### Request Body Format

Same as schema registration:
```json
{
  "schema": "{\"type\":\"record\",...}",
  "schemaType": "AVRO",
  "references": []
}
```

Reuse `SchemaRequest` from `subjects.rs` or define `CompatibilityTestRequest` (same shape).

### Effective Compatibility Resolution

Need `get_effective_compatibility(pool, subject)` — subject-level then global fallback:
```rust
pub async fn get_effective_compatibility(pool: &PgPool, subject: &str) -> Result<String, sqlx::Error> {
    if let Some(level) = get_subject_level(pool, subject).await? {
        Ok(level)
    } else {
        get_global_level(pool).await
    }
}
```

### Files Modified

| File | Changes |
|------|---------|
| `src/schema/mod.rs` | `CompatDirection` enum, `CompatibilityResult` struct, `check_with_direction` shared helper, dispatch |
| `src/schema/avro.rs` | `check_compatibility` using `apache_avro::SchemaCompatibility`, error chain extraction |
| `src/schema/json_schema/mod.rs` | Parse + canonical (split from flat file) |
| `src/schema/json_schema/diff.rs` | Full Confluent-compatible diff engine (111 types, 57 compatible set) |
| `src/schema/protobuf/mod.rs` | Parse + canonical (split from flat file) |
| `src/schema/protobuf/diff.rs` | Full Confluent-compatible diff engine (25 types, 14 compatible set) |
| `src/storage/compatibility.rs` | `get_effective_compatibility` (subject → global fallback) |
| `src/api/compatibility.rs` | `CompatibilityTestRequest`, `CompatibilityTestParams`, two test handlers, `resolve_version` helper |
| `src/api/mod.rs` | 2 POST routes under `/compatibility/subjects/` |
| `tests/common/api.rs` | `test_compatibility`, `check_compatibility` helpers |
| `tests/common/mod.rs` | `COMPAT_AVRO_*` fixtures |
| `tests/api_compatibility_test.rs` | 95 integration tests (14 Avro + 62 JSON Schema + 19 Protobuf) |
| `Cargo.toml` | Added `prost-types`, `regex` dependencies |

### Previous Story Intelligence

From Story 4.1:
- Compatibility config CRUD exists: GET/PUT/DELETE on /config and /config/{subject}
- `CompatibilityRequest` struct accepts `compatibility` field
- Valid levels: BACKWARD, FORWARD, FULL, NONE, BACKWARD_TRANSITIVE, FORWARD_TRANSITIVE, FULL_TRANSITIVE
- Default global level: BACKWARD

From Story 4.3:
- `get_effective_normalize(pool, subject)` pattern: subject → global fallback
- `normalize` param resolves via `params.normalize OR get_effective_normalize(subject)`
- Same pattern needed for compatibility level resolution

### Test Schema Fixtures for Compatibility

Existing fixtures in `tests/common/mod.rs`:
- `AVRO_SCHEMA_V1`: `{record Test, fields: [id:int]}` — base schema
- `AVRO_SCHEMA_V2`: `{record Test, fields: [id:int, name:string]}` — adds optional field (BACKWARD compatible)
- `AVRO_SCHEMA_V3`: `{record Test, fields: [id:int, name:string, active:boolean]}` — adds another field

V1 → V2 is BACKWARD compatible (V2 can read V1 data — name defaults to null/empty)
V2 → V1 is FORWARD compatible (V1 can read V2 data — ignores name field)

May need additional fixtures for incompatible cases (removing required field, changing field type).

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 5.1] — AC definitions
- [Source: _bmad-output/planning-artifacts/prd.md#FR20,FR37,FR38] — Functional requirements
- [Source: _bmad-output/planning-artifacts/architecture.md] — SchemaHandler trait design, compatibility modes
- [Source: src/api/compatibility.rs] — Existing config handlers, CompatibilityRequest
- [Source: src/storage/compatibility.rs] — get_subject_level, get_global_level, get_effective_normalize pattern
- [Source: src/schema/avro.rs] — Current parse-only implementation, apache-avro crate
- [Source: src/error.rs] — IncompatibleSchema (40901) already defined

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6 (1M context)

### Debug Log References

### Completion Notes List

- **Task 1**: Full Confluent-compatible compatibility checking for all 3 formats:
  - **Avro**: `apache_avro::SchemaCompatibility::can_read`/`mutual_read` with recursive error chain extraction
  - **JSON Schema**: Complete diff engine (111 DiffType variants, 57 COMPATIBLE_CHANGES_STRICT). Covers type promotion (integer→number), string/number/object/array/enum/combined constraints, dependencies, $ref resolution, const, not reversal, FalseSchema/EmptySchema, singleton unwrapping, partially open content model with regex patternProperties, tuple items (items + prefixItems), max bipartite matching for combined subschemas, multipleOf divisibility
  - **Protobuf**: Complete diff engine (25 DiffType variants, 14 COMPATIBLE_CHANGES). Covers package/message/field/enum/oneof comparison, field type classification (scalar vs message/enum), label changes (string/bytes vs numeric), required field detection, oneof move detection (single vs multiple), nested recursion
  - Shared `CompatDirection` enum with `from_level()` for BACKWARD/FORWARD/FULL/NONE
  - Shared `check_with_direction()` eliminates direction logic duplication between JSON Schema and Protobuf
  - Refactored `json_schema.rs` and `protobuf.rs` from flat files to directory modules (`mod.rs` + `diff.rs`)
- **Task 2**: Added `get_effective_compatibility(pool, subject)` — subject → global fallback
- **Task 3**: Two handlers: `test_compatibility_by_version` (specific/latest) and `test_compatibility_against_all_versions`. `resolve_version` helper, verbose mode, `CompatibilityTestRequest`/`CompatibilityTestParams`
- **Task 4**: 95 integration tests: 14 Avro (endpoint behavior: direction, verbose, all-versions, error codes) + 62 JSON Schema (every diff category) + 19 Protobuf (every diff category). Test helpers `test_compatibility`/`check_compatibility` in `common/api.rs`

### Change Log

- 2026-04-13: Initial implementation — endpoints + Avro compat via apache-avro
- 2026-04-13: Added full JSON Schema + Protobuf diff engines matching Confluent source code
- 2026-04-13: Fixed multipleOf (divisibility not numeric comparison), not-schema reversal, singleton unwrap, FalseSchema/EmptySchema, tuple items (prefixItems for draft 2020-12), ItemRemovedFromClosedContentModel
- 2026-04-13: Refactored json_schema.rs → json_schema/mod.rs + diff.rs, protobuf.rs → protobuf/mod.rs + diff.rs
- 2026-04-13: Extracted check_with_direction to mod.rs (eliminates direction logic duplication)
- 2026-04-14: Added 81 integration tests (62 JSON Schema + 19 Protobuf) covering every diff category. Moved test helpers to common/api.rs. Removed local helpers from test file. 276 total tests pass.
- 2026-04-14: Confluent conformance — extracted all test fixtures from Confluent Schema Registry repo and replayed them against our diff engines. 100% pass rate on all replicable test cases:
  - JSON Schema: 251 fixture cases (4 files) + 2 inline tests + 2 circular ref tests = 255 cases
  - Protobuf: 43 fixture cases + 1 inline test = 44 cases
  - Total: 299 Confluent conformance cases passing
- 2026-04-14: JSON Schema diff engine fixes for Confluent conformance:
  - Type comparison: treat absent `type` as empty set, not "skip"
  - Singleton unwrap: added `allOf` + re-check combined transitions after unwrap
  - Combined keyword comparison: bipartite matching for sub-diff pairing instead of positional zip, `CombinedTypeSubschemasChanged` detection, length-based extended/narrowed
  - Combined criteria change: fixed EXTENDED vs CHANGED condition for allOf target
  - Non-combined → allOf transition: require ALL subschemas compatible (intersection semantics)
  - Combined → non-combined: use `SumTypeNarrowed` for oneOf/anyOf (was always `ProductTypeNarrowed`)
  - Tuple items: check NEW schema's model for removals, actual coverage check for additions in partially open models
  - `is_covered_by_partial_model`: fixed comparison direction for property removals
  - Dependencies: added `dependentRequired`/`dependentSchemas` support (2020-12)
  - Array items: handle boolean `items` true/false (2020-12)
  - Empty→constrained schema transition: detect as incompatible (Confluent `testSchemaAddsProperties`)
  - Kafka Connect `connect.type=bytes`: skip type comparison when both schemas have this extension
- 2026-04-14: Protobuf diff engine fixes for Confluent conformance:
  - `resolve_field_type`: exact qualified match instead of greedy `ends_with(short)`
  - `detect_oneof_field_moves`: detect moves to renamed oneofs (check field membership, not oneof name)
  - `type_names_equal`: fully-qualified names compared exactly (no short-name fallback when both start with `.`)
  - Import change detection: heuristic for external type flagging when no dependencies provided
  - Dependency resolution: `TypeRegistry` parses imported `.proto` files, `compare_external_type_refs` resolves and compares external message definitions recursively
  - `check_with_deps` / `check_compatibility_with_deps`: extended API accepting dependency proto content

### File List

- src/schema/mod.rs (modified) — `CompatDirection`, `CompatibilityResult`, `check_with_direction` shared helper, dispatch
- src/schema/avro.rs (modified) — `check_compatibility` via apache-avro, `collect_error_messages`
- src/schema/json_schema/mod.rs (new, replaces json_schema.rs) — parsing + canonical form
- src/schema/json_schema/diff.rs (new) — Confluent-compatible diff engine (111 types, 57 compatible set, bipartite matching)
- src/schema/protobuf/mod.rs (new, replaces protobuf.rs) — parsing + canonical form + `check_compatibility_with_deps`
- src/schema/protobuf/diff.rs (new) — Confluent-compatible diff engine (25 types, 14 compatible set, `TypeRegistry` for dependency resolution)
- src/storage/compatibility.rs (modified) — `get_effective_compatibility`
- src/api/compatibility.rs (modified) — test handlers, request/param structs, `resolve_version`
- src/api/mod.rs (modified) — 2 POST routes
- tests/common/api.rs (modified) — `test_compatibility`, `check_compatibility` helpers
- tests/common/mod.rs (modified) — `COMPAT_AVRO_*` fixtures
- tests/api_compatibility_test.rs (new) — 14 integration tests (Avro endpoint behavior: direction, verbose, all-versions, error codes)
- tests/confluent_json_schema_compat.rs (new) — 8 tests replaying Confluent's `SchemaDiffTest.java` + `CircularRefSchemaDiffTest.java` (255 cases)
- tests/confluent_protobuf_compat.rs (new) — 2 tests replaying Confluent's `SchemaDiffTest.java` (44 cases)
- tests/fixtures/confluent/ (new) — 5 JSON fixture files extracted from Confluent Schema Registry repo (byte-identical to source)
- Cargo.toml (modified) — added `prost-types`, `regex`
