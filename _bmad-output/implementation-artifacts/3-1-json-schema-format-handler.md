# Story 3.1: JSON Schema Format Handler

Status: done

## Story

As a **developer**,
I want to register JSON Schema schemas,
so that I can use JSON Schema for data validation in my pipeline.

## Acceptance Criteria

1. **Given** a valid JSON Schema document
   **When** I send `POST /subjects/{subject}/versions` with `{"schemaType": "JSON", "schema": "<json_schema>"}`
   **Then** I receive HTTP 200 with the assigned schema ID
   **And** the schema is parsed, validated, and stored with its canonical form

2. **Given** an invalid JSON Schema document
   **When** I send `POST /subjects/{subject}/versions` with `{"schemaType": "JSON", "schema": "<invalid>"}`
   **Then** I receive HTTP 422 with Confluent error code 42201

3. **Given** a registered JSON Schema
   **When** I retrieve it via `GET /schemas/ids/{id}`
   **Then** the response includes `"schemaType": "JSON"`

**FRs Covered:** FR16

## Tasks / Subtasks

- [x] Task 1: Add jsonschema dependency (AC: #1)
  - [x] Add `jsonschema = "0.45.1"` (no default features) to Cargo.toml
  - [x] Add `sha2 = "0.10"` for fingerprinting

- [x] Task 2: Create JSON Schema parser module (AC: #1, #2)
  - [x] Create `src/schema/json_schema.rs`
  - [x] Validate with `jsonschema::meta::is_valid`
  - [x] Canonical form: sorted-key deterministic JSON
  - [x] Fingerprint: SHA-256 hex of canonical form

- [x] Task 3: Extend SchemaFormat enum (AC: #1, #2, #3)
  - [x] Add `Json` variant, update from_optional, as_str, parse dispatch

- [x] Task 4: Integration tests (AC: #1, #2, #3)
  - [x] 5 integration tests in `tests/api_json_schema.rs`
  - [x] JSON Schema fixtures in `tests/common/mod.rs`
  - [x] `register_schema_with_type` helper in `tests/common/api.rs`

- [x] Task 5: Unit tests for json_schema parser
  - [x] 6 unit tests (valid, invalid JSON, invalid schema, fingerprint stability, different fingerprints, key sorting)

- [x] Task 6: Verify all tests pass
  - [x] `cargo clippy` — zero warnings (pedantic)
  - [x] `cargo test` — 89 tests pass (78 existing + 11 new)

## Dev Notes

### JSON Schema Canonical Form

No standard canonical form exists for JSON Schema (unlike Avro which has a spec-defined canonical form). We use deterministic JSON serialization: parse the JSON, then re-serialize with `serde_json` using sorted keys. This is consistent with how Confluent handles JSON Schema canonical form.

### Fingerprinting

Avro uses Rabin (64-bit CRC). For JSON Schema, we use SHA-256 of the canonical form, hex-encoded. This matches Confluent's approach for non-Avro formats. The fingerprint column in the DB is TEXT so it accepts any format.

### Validation Strategy

The `jsonschema` crate provides `jsonschema::meta::is_valid()` which validates that a JSON document is a valid JSON Schema. We don't need to compile the schema — just validate its structure.

Note: Check latest API for `jsonschema` crate — the API may have changed since architecture doc was written (was 0.45.0, latest may differ).

### Existing Patterns

- **avro.rs**: `src/schema/avro.rs` — exact same pattern: `parse(raw) -> Result<ParsedSchema, KoraError>`
- **SchemaFormat**: `src/schema/mod.rs` — add variant, update match arms
- **Test pattern**: `tests/api_register_schema.rs` — integration test pattern

### Architecture Compliance

- One file per format: `src/schema/json_schema.rs`
- Same `ParsedSchema` struct (canonical_form + fingerprint)
- Enum dispatch, no dynamic dispatch
- `#![deny(missing_docs)]` — document everything

### References

- [Source: epics.md — Epic 3, Story 3.1]
- [Source: prd.md — FR16]
- [Source: architecture.md — jsonschema crate, schema format abstraction]
- [Source: 2-4-schema-references-and-dependency-protection.md — latest patterns]

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6 (1M context)

### Completion Notes List

- Added jsonschema 0.45.1 + sha2 0.10 dependencies
- Created src/schema/json_schema.rs: parse, canonical_json, meta-validation, SHA-256 fingerprint
- Extended SchemaFormat with Json variant, updated dispatch
- Canonical form uses sorted-key recursive JSON serialization
- 5 integration tests + 6 unit tests, 89 total passing

### File List

**New:**
- `src/schema/json_schema.rs`
- `tests/api_json_schema.rs`

**Modified:**
- `Cargo.toml` — added jsonschema, sha2
- `Cargo.lock` — updated
- `src/schema/mod.rs` — Json variant, dispatch
- `tests/common/mod.rs` — JSON Schema fixtures
- `tests/common/api.rs` — register_schema_with_type helper

### Change Log

- 2026-04-09: Story 3.1 implemented — JSON Schema format handler (FR16)
