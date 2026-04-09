# Story 3.2: Protobuf Format Handler

Status: done

## Story

As a **developer**,
I want to register Protobuf schemas,
so that I can use Protobuf for high-performance serialization in my pipeline.

## Acceptance Criteria

1. **Given** a valid `.proto` definition as a string
   **When** I send `POST /subjects/{subject}/versions` with `{"schemaType": "PROTOBUF", "schema": "<proto_def>"}`
   **Then** I receive HTTP 200 with the assigned schema ID
   **And** the schema is parsed, validated, and stored with its canonical form

2. **Given** an invalid Protobuf definition
   **When** I send `POST /subjects/{subject}/versions` with `{"schemaType": "PROTOBUF", "schema": "<invalid>"}`
   **Then** I receive HTTP 422 with Confluent error code 42201

3. **Given** a registered Protobuf schema
   **When** I retrieve it via `GET /schemas/ids/{id}`
   **Then** the response includes `"schemaType": "PROTOBUF"`

**FRs Covered:** FR17

## Tasks / Subtasks

- [x] Task 1: Add protobuf parsing dependency (AC: #1)
  - [x] Add `protox-parse = "0.9"` to Cargo.toml

- [x] Task 2: Create Protobuf parser module (AC: #1, #2)
  - [x] Create `src/schema/protobuf.rs`
  - [x] Validate with `protox_parse::parse()`, canonical form via whitespace normalization, SHA-256 fingerprint

- [x] Task 3: Extend SchemaFormat enum (AC: #1, #2, #3)
  - [x] Add `Protobuf` variant, update from_optional, as_str, parse dispatch

- [x] Task 4: Integration tests in api_register_schema.rs (AC: #1, #2, #3)
  - [x] Protobuf fixtures (PROTO_SCHEMA_V1, V2) in tests/common/mod.rs
  - [x] register_protobuf_schema_valid_succeeds (AC #1)
  - [x] register_protobuf_schema_invalid_returns_422 (AC #2)
  - [x] register_protobuf_schema_retrieve_includes_type (AC #3)
  - [x] register_protobuf_schema_idempotent_returns_same_id
  - [x] register_protobuf_schema_listed_under_versions

- [x] Task 5: Unit tests in schema_parsing.rs (AC: #1, #2)
  - [x] protobuf_parse_valid
  - [x] protobuf_parse_invalid
  - [x] protobuf_canonical_form_is_stable
  - [x] protobuf_fingerprint_is_stable
  - [x] protobuf_different_schemas_have_different_fingerprints
  - [x] format_protobuf_accepted

- [x] Task 6: Verify all tests pass
  - [x] `cargo clippy` — zero warnings (pedantic)
  - [x] `cargo test` — 102 tests pass (91 existing + 11 new)

## Dev Notes

### Protobuf Schema as String

Confluent sends the raw `.proto` definition as a JSON string:
```json
{"schemaType": "PROTOBUF", "schema": "syntax = \"proto3\";\nmessage Test { int32 id = 1; }"}
```

### Library Choice: protox-parse vs protobuf-parse

- Architecture doc suggests `prost` + `prost-build` — but those are compile-time tools, not runtime parsers
- `protox-parse` (0.9): lightweight runtime .proto parser, returns AST. No proto compilation needed.
- `protobuf-parse` (3.7): heavier, returns FileDescriptor. More than we need.
- Choice: `protox-parse` — just validates syntax and gives us the AST for canonical form

### Canonical Form

No standard canonical form for Protobuf. Approach:
- Parse proto text → AST
- Re-serialize from AST in deterministic order
- Or: simpler approach — normalize whitespace, trim, and hash the raw text (Confluent's approach for deduplication)
- Start with the simpler approach: trim + normalize whitespace for canonical form

### Fingerprint

Same as JSON Schema: SHA-256 hex of canonical form. Already using `sha2` crate.

### Existing Patterns

- **json_schema.rs**: exact same pattern — `parse(raw) -> Result<ParsedSchema, KoraError>`
- **SchemaFormat**: `src/schema/mod.rs` — add variant, same as Json was added
- **Test naming**: follow `register_protobuf_*` / `protobuf_*` convention (matches avro/json)
- **Test sections**: add `// -- Protobuf --` section after JSON in api_register_schema.rs
- **Fixtures**: add to tests/common/mod.rs with `// -- Protobuf Fixtures --` section

### Architecture Compliance

- One file per format: `src/schema/protobuf.rs`
- Same `ParsedSchema` struct (canonical_form + fingerprint)
- Enum dispatch, no dynamic dispatch
- Tests organized by concern, format-specific tests clearly labeled

### References

- [Source: epics.md — Epic 3, Story 3.2]
- [Source: prd.md — FR17]
- [Source: architecture.md — schema format abstraction]
- [Source: 3-1-json-schema-format-handler.md — patterns and approach]

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6 (1M context)

### Completion Notes List

- Added protox-parse 0.9.0 dependency for runtime .proto parsing
- Created src/schema/protobuf.rs: parse, canonical_proto (whitespace normalization), SHA-256 fingerprint
- Extended SchemaFormat with Protobuf variant, updated dispatch
- 4 integration tests in api_register_schema.rs (valid, invalid, retrieve type, idempotent)
- 5 unit tests in schema_parsing.rs (parse valid/invalid, fingerprint stable/different, format accepted)
- 100 tests total, all passing, zero clippy warnings

### File List

**New:**
- `src/schema/protobuf.rs`

**Modified:**
- `Cargo.toml` — added protox-parse
- `Cargo.lock` — updated
- `src/schema/mod.rs` — Protobuf variant, dispatch
- `tests/api_register_schema.rs` — Protobuf section
- `tests/schema_parsing.rs` — Protobuf parsing + format_protobuf_accepted
- `tests/common/mod.rs` — Protobuf fixtures

### Change Log

- 2026-04-09: Story 3.2 implemented — Protobuf format handler (FR17)
- 2026-04-09: Code review fixes — added canonical_form_is_stable + listed_under_versions tests, fixed false→ACTIVE_ONLY constant
