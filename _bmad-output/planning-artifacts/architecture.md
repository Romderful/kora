---
stepsCompleted: [1, 2, 3, 4, 5, 6, 7, 8]
inputDocuments: ["_bmad-output/planning-artifacts/prd.md", "_bmad-output/planning-artifacts/product-brief-kora.md"]
workflowType: 'architecture'
lastStep: 8
status: 'complete'
completedAt: '2026-04-04'
project_name: 'kora'
user_name: 'Mindsky'
date: '2026-04-04'
---

# Architecture Decision Document

_This document builds collaboratively through step-by-step discovery. Sections are appended as we work through each architectural decision together._

## Project Context Analysis

### Requirements Overview

**49 Functional Requirements** across 9 capability areas:

| Capability Area | FR Count | Complexity | Notes |
|---|---|---|---|
| Schema Management | 8 | Medium | CRUD + versioning + ID allocation |
| Schema Formats | 5 | High | Avro, JSON Schema, Protobuf — each with own parser |
| Compatibility | 8 | High | 7 modes × 3 formats, verbose incompatibility messages |
| Storage & Persistence | 5 | Medium | PostgreSQL, dual-mode (embedded Docker / external) |
| Registry Configuration | 4 | Low | Global + subject-level mode/compatibility |
| Observability | 5 | Medium | Prometheus metrics, structured logging |
| API Conformance | 5 | Medium | Confluent wire-compatibility, error contract |
| Operational | 3 | Low | Health check, graceful shutdown, config via env |

**Non-Functional Requirements Summary:**

- **Performance**: P99 < 5ms reads, < 20ms writes, < 100ms compatibility checks. 10K+ schemas supported.
- **Reliability**: Zero data loss, graceful degradation, atomic operations.
- **Scalability**: Single-instance MVP, multi-instance via external PG for HA.
- **Integration**: Confluent client wire-compatible, standard Prometheus exposition.

### Technical Constraints & Dependencies

**Hard Constraints (from PRD):**

| Constraint | Source | Impact |
|---|---|---|
| Rust | Product Brief | Language for all components |
| PostgreSQL | PRD FR-ST-01 | Single storage backend |
| Confluent SR API | PRD FR-AC-01 | Wire-compatible REST API — exact endpoint paths, response shapes, error codes |
| 3 Schema Formats | PRD FR-SF-01/02/03 | Avro, JSON Schema, Protobuf — each needs a dedicated parser |
| Docker all-in-one | PRD User Journey (Nadia) | Embedded PG in container, auto-detected via DATABASE_URL |
| No auth in MVP | PRD Scope | Simplifies middleware, but must not preclude future addition |

**Derived Constraints:**

- Global sequential schema IDs → requires coordination strategy (PG sequence in MVP, may need distributed approach later)
- Confluent error contract → every error response must match exact Confluent format (error_code + message)
- Binary distribution → single static binary + Docker image

### Cross-Cutting Concerns

**1. Schema Format Abstraction**
Three distinct parsers (Avro, JSON Schema, Protobuf) must present a unified interface for: parsing/validation, canonical form normalization, and compatibility checking. This is the highest-complexity architectural concern — each format has its own rules for compatibility.

**2. Confluent API Error Contract**
Every endpoint must return errors in the exact Confluent format. This means a centralized error mapping layer that translates internal errors to Confluent error codes (40401, 40402, 40403, 42201, 42202, 40901, 50001, etc.).

**3. Caching Strategy**
Schemas are append-mostly with rare deletions — highly cacheable with simple write-through invalidation on mutations. Hard deletes invalidate cache entries; soft deletes mark as 404. Confluent SR also refuses hard deletes if a schema is referenced by others, providing additional stability. The cache layer targets P99 < 5ms for read-heavy workloads (schema lookups by ID dominate production traffic from deserializers).

**4. Global Sequential ID Allocation**
Schema IDs must be globally unique and sequential. In single-instance MVP, a PostgreSQL sequence handles this. For future multi-instance HA, this becomes a coordination challenge (but explicitly out of MVP scope).

**5. Schema References & Deletion Protection**
Schemas can reference other schemas (particularly Protobuf imports and JSON Schema `$ref`). Hard deletes must be refused if a schema is referenced by another — this creates a dependency graph that must be tracked and queried efficiently.

## Starter Template Evaluation

### Primary Technology Domain

Rust API/Backend — REST API service with PostgreSQL storage. No starter template CLI exists in the Rust ecosystem; project initialized with `cargo init` and composed from individual crates.

### Foundation Crate Selection

| Layer | Crate | Version | Rationale |
|---|---|---|---|
| Runtime | `tokio` | 1.51.0 | De facto async runtime, LTS release |
| Web Framework | `axum` | 0.8.8 | Tokio-native, tower middleware, macro-free routing, 278M downloads |
| Database | `sqlx` | 0.8.6 | Async, compile-time checked queries, PG-native, built-in pool + migrations |
| Serialization | `serde` + `serde_json` | — | Non-negotiable standard |
| Avro | `apache-avro` | 0.21.0 | Official Apache crate. Built-in `SchemaCompatibility::can_read()`, canonical form, fingerprinting |
| Protobuf | `prost` + `prost-build` | 0.14.3 | Tokio ecosystem, idiomatic Rust from `.proto` files |
| JSON Schema | `jsonschema` | 0.45.0 | Draft 4–2020-12 support, high performance (up to 645x faster than alternatives) |
| Logging | `tracing` + `tracing-subscriber` | 0.1.44 | Structured async-aware diagnostics, tokio ecosystem |
| Metrics | `metrics` + `metrics-exporter-prometheus` | 0.24.3 | Lightweight facade with Prometheus exporter |
| Config | `figment` | 0.10.19 | Layered config (defaults → file → env vars), serde integration |
| Error Handling | `thiserror` | — | Derive macro for custom error types |

### Architectural Decisions Provided by Stack

**Ecosystem Coherence:** axum + sqlx + prost + tracing are all tokio-rs projects sharing runtime, patterns, and release cadence.

**Database Access Pattern:** sqlx with compile-time checked SQL (not an ORM). Raw SQL queries verified against PG at build time — ideal for precise control over schema storage queries.

**Schema Format Strategy:**
- **Avro**: `apache-avro` provides parsing, compatibility checking, canonical form, and fingerprinting out of the box.
- **JSON Schema**: `jsonschema` provides validation. Compatibility checking requires custom implementation.
- **Protobuf**: `prost` handles parsing. Compatibility checking requires custom implementation based on wire-compatibility rules.

**Observability:** `tracing` for structured logs + `metrics` for Prometheus exposition. Decoupled systems.

### Initialization Command

```bash
cargo init kora && cd kora
cargo add axum tokio --features tokio/full
cargo add sqlx --features "runtime-tokio,tls-rustls-ring-webpki,postgres,migrate,macros,uuid,chrono,json"
cargo add serde serde_json --features serde/derive
cargo add apache-avro
cargo add prost prost-types
cargo add jsonschema --default-features=false
cargo add tracing tracing-subscriber --features tracing-subscriber/json
cargo add metrics metrics-exporter-prometheus
cargo add figment --features "toml,env"
cargo add thiserror
cargo add uuid --features "v4,serde"
```

**Note:** Project initialization using these commands should be the first implementation story.

## Core Architectural Decisions

### Code Philosophy

**Rule: No bullshit. No over-engineering. Simple, readable, documented Rust.**

1. **Simple** — if it's not necessary, it's gone
2. **Readable** — code reads like Rust prose. Explicit names, short functions
3. **Documented** — `#![deny(missing_docs)]` at crate level. Every public item documented
4. **No premature abstraction** — no trait "just in case", no generics for fun, no builder pattern when a constructor suffices
5. **Clippy pedantic** — `#![deny(clippy::pedantic)]`
6. **No architectural astronautics** — no hexagonal, no DDD layers, no ports-and-adapters. Clean Rust modules that do what they say

### Data Architecture

**Schema Storage Model:** Two-table design for global content dedup (Confluent-compatible). `schema_contents` stores unique schema content (text, canonical form, fingerprints) with a global sequential ID. `schema_versions` maps (subject, version) to a content ID, with soft-delete tracking. Identical schema text registered under different subjects shares the same content ID. Format-specific logic lives in Rust code, not in DB schema.

**Schema ID Allocation:** PostgreSQL `BIGSERIAL` sequence on `schema_contents`. The content ID is the global schema ID returned by the API. Same content under different subjects shares one ID (Confluent behavior).

**Migrations:** sqlx built-in migrations (compile-time embedded).

**Cache:** In-process `DashMap` for concurrent reads. Schema-by-ID is the hot path (deserializers call this constantly). Single-instance MVP = no cache coherence problem. Write-through invalidation on mutations.

### API & Communication

**Error Handling:** Single `KoraError` enum implementing axum's `IntoResponse`. Maps internal errors to Confluent error codes (40401, 40402, 42201, etc.). One place, one mapping.

**Request/Response Format:** Confluent JSON wire format. `serde_json::Value` for flexible schema content, typed structs for API envelopes.

**API Versioning:** None — Confluent API has no version prefix.

**Content Type:** Accept and return `application/vnd.schemaregistry.v1+json` (with `application/json` fallback).

### Schema Format Abstraction

**Trait Design:** Single `SchemaHandler` trait with methods: `parse`, `canonical_form`, `check_compatibility`, `validate_references`. Each format (Avro, JSON Schema, Protobuf) implements this one trait.

**Format Dispatch:** Enum dispatch over three known variants. No dynamic dispatch, no runtime plugins. The compiler checks exhaustiveness.

### Infrastructure & Deployment

**Binary Distribution:** Single static binary (`cargo build --release`) + Docker image with embedded PG (multi-stage build, s6-overlay for process supervision).

**Config Layering:** figment: defaults → optional config file → env vars. `KORA_` prefix for Kora-specific settings, `DATABASE_URL` for PG connection.

**Health Check:** `GET /health` with PG connectivity check. If DB is down, service is unhealthy.

**Graceful Shutdown:** tokio signal handler + connection draining. Standard axum pattern.

### Decision Impact Analysis

**Implementation Sequence:**
1. Project scaffold + DB migrations + health check
2. Schema storage (CRUD) + ID allocation
3. Avro format handler (parse, canonical, compatibility)
4. Core API endpoints (subjects, schemas)
5. JSON Schema + Protobuf handlers
6. Cache layer
7. Observability (metrics + structured logging)
8. Docker all-in-one packaging

**Deferred Decisions (Post-MVP):**
- Authentication/authorization
- Rate limiting
- Multi-instance cache coherence
- Distributed ID allocation

## Implementation Patterns & Consistency Rules

### Naming Patterns

**Database:**
- Tables: `snake_case`, plural — `schemas`, `subjects`, `schema_references`
- Columns: `snake_case` — `schema_id`, `created_at`, `schema_type`
- Indexes: `idx_{table}_{columns}` — `idx_schemas_subject_version`
- Foreign keys: `fk_{table}_{ref_table}` — `fk_schema_references_schemas`

**API:**
- Confluent paths exactly as spec'd: `/subjects`, `/schemas/ids/{id}`, `/compatibility/subjects/{subject}/versions/{version}`
- All endpoints match Confluent Schema Registry API paths exactly (no custom extensions)
- Query params: `snake_case` — matching Confluent where applicable (`deleted=true`)

**Rust Code:**
- Modules: `snake_case` — `schema_handler`, `compatibility`
- Structs/Enums: `PascalCase` — `SchemaHandler`, `CompatibilityMode`
- Functions: `snake_case` — `check_compatibility`, `get_schema_by_id`
- Constants: `SCREAMING_SNAKE` — `DEFAULT_COMPATIBILITY_MODE`
- No abbreviations except universally understood ones (`id`, `db`, `pg`)

### Structure Patterns

**Module Organization:**
```
src/
  main.rs           — entrypoint, server setup
  lib.rs            — crate root, re-exports
  config.rs         — figment config struct
  error.rs          — KoraError enum + IntoResponse
  api/              — axum handlers, one file per resource
    mod.rs
    subjects.rs
    schemas.rs
    compatibility.rs
    config.rs       — registry config endpoints
    health.rs
  storage/          — sqlx queries, one file per concern
    mod.rs
    schemas.rs
    subjects.rs
  schema/           — format parsing + compatibility
    mod.rs
    handler.rs      — SchemaHandler trait + dispatch enum
    avro.rs
    json_schema.rs
    protobuf.rs
  cache.rs          — DashMap cache layer
  metrics.rs        — Prometheus metric definitions
```

**Tests:** Co-located in same file (`#[cfg(test)] mod tests`). Integration tests in top-level `tests/` directory.

### Format Patterns

**API Responses:**
- Success: direct Confluent-format JSON (no wrapper). `{"subject":"test","id":1,"version":1,"schema":"{...}"}`
- Errors: exact Confluent format: `{"error_code": 40401, "message": "Subject not found"}`
- HTTP status codes match Confluent mapping (404 for 40401/40402, 422 for 42201, 409 for 40901, 500 for 50001)

**JSON Fields:** `snake_case` everywhere — matches Confluent's format (`schema_type`, `error_code`).

**Dates:** ISO 8601 strings in API. `chrono::DateTime<Utc>` in Rust, `TIMESTAMPTZ` in PG.

### Process Patterns

**Error Flow:**
1. Storage/schema functions return `Result<T, KoraError>`
2. `KoraError` variants map 1:1 to Confluent error codes
3. axum `IntoResponse` impl produces the JSON error body + HTTP status
4. No panics. No `unwrap()` in non-test code. `expect()` only for provably-safe cases with a message explaining why

**Logging:**
- `tracing::info!` for request lifecycle (received, completed)
- `tracing::warn!` for recoverable issues (cache miss, slow query)
- `tracing::error!` for actual failures (DB down, parse error)
- Always structured: `tracing::info!(subject = %subject, version = %version, "schema registered")`

**Validation:**
- Validate at the API boundary (handlers). Storage functions assume valid input
- Schema parsing = validation. If `SchemaHandler::parse` succeeds, the schema is valid

### Development Process

**TDD — Non-negotiable. Every feature follows Red-Green-Refactor:**
1. **Red:** Write a failing test that defines the expected behavior
2. **Green:** Write the minimum code to make the test pass
3. **Refactor:** Clean up while keeping tests green

No production code without a test that drove its creation. Tests are written FIRST, not after.

### Enforcement Rules

**All code MUST:**
- Follow TDD: test first, code second, refactor third
- Pass `cargo clippy -- -D clippy::all -D clippy::pedantic`
- Pass `cargo test`
- Have `#![deny(missing_docs)]` at crate level
- Have zero `unwrap()` outside `#[cfg(test)]`

## Project Structure & Boundaries

### Complete Project Directory Structure

```
kora/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── .env.example                    — DATABASE_URL, KORA_ vars documented
├── .gitignore
├── Dockerfile                      — multi-stage: build + runtime with s6 + PG
├── docker-compose.yml              — dev setup (PG only)
├── migrations/
│   └── 001_initial_schema.sql      — schemas, subjects, config, references tables
├── src/
│   ├── main.rs                     — tokio entrypoint, server bind, graceful shutdown
│   ├── lib.rs                      — #![deny(missing_docs, clippy::pedantic)], crate docs
│   ├── config.rs                   — KoraConfig struct (figment), DATABASE_URL, port, log level
│   ├── error.rs                    — KoraError enum, Confluent error code mapping, IntoResponse
│   ├── cache.rs                    — DashMap<SchemaId, Arc<StoredSchema>>, get/put/invalidate
│   ├── metrics.rs                  — counter!/histogram! definitions, /metrics endpoint
│   ├── api/
│   │   ├── mod.rs                  — Router construction, all route registration
│   │   ├── subjects.rs             — GET/POST /subjects, /subjects/{subject}/versions
│   │   ├── schemas.rs              — GET /schemas/ids/{id}, /schemas/ids/{id}/schema
│   │   ├── compatibility.rs        — POST /compatibility/subjects/{subject}/versions/{version}
│   │   ├── config_endpoints.rs     — GET/PUT /config, /config/{subject}
│   │   ├── mode.rs                 — GET/PUT /mode, /mode/{subject}
│   │   ├── health.rs               — GET /health (PG check)
│   │   └── schemas_list.rs         — GET /schemas (list all schemas with filters)
│   ├── storage/
│   │   ├── mod.rs                  — PgPool setup, connection helper
│   │   ├── schemas.rs              — insert/get/delete/list schemas, ID allocation
│   │   ├── subjects.rs             — subject CRUD, version listing, soft/hard delete
│   │   ├── config.rs               — global + per-subject compatibility/mode settings
│   │   └── references.rs           — schema reference tracking, dependency graph queries
│   └── schema/
│       ├── mod.rs                  — SchemaFormat enum, dispatch to handlers
│       ├── handler.rs              — SchemaHandler trait definition
│       ├── avro.rs                 — Avro: parse, canonical_form, compatibility
│       ├── json_schema.rs          — JSON Schema: parse, canonical_form, compatibility
│       └── protobuf.rs             — Protobuf: parse, canonical_form, compatibility
├── tests/
│   ├── common/
│   │   └── mod.rs                  — test DB setup, test server helper, fixtures
│   ├── api_subjects.rs             — subject endpoint integration tests
│   ├── api_schemas.rs              — schema CRUD integration tests
│   ├── api_compatibility.rs        — compatibility check integration tests
│   └── confluent_wire_compat.rs    — wire-compatibility tests against Confluent spec
```

### Requirements to Structure Mapping

| FR Category | Primary Files | Key FRs |
|---|---|---|
| Schema Management | `api/subjects.rs`, `api/schemas.rs`, `storage/schemas.rs`, `storage/subjects.rs` | FR-SM-01 through FR-SM-08 |
| Schema Formats | `schema/avro.rs`, `schema/json_schema.rs`, `schema/protobuf.rs` | FR-SF-01/02/03 |
| Compatibility | `api/compatibility.rs`, `schema/handler.rs` | FR-CP-01 through FR-CP-08 |
| Storage | `storage/mod.rs`, `storage/schemas.rs`, `migrations/` | FR-ST-01 through FR-ST-05 |
| Registry Config | `api/config_endpoints.rs`, `api/mode.rs`, `storage/config.rs` | FR-RC-01 through FR-RC-04 |
| Observability | `metrics.rs`, `main.rs` | FR-OB-01 through FR-OB-05 |
| API Conformance | `error.rs`, `api/mod.rs` | FR-AC-01 through FR-AC-05 |

### Architectural Boundaries

**API → Storage:** Handlers call storage functions directly. No intermediate "service" layer. Storage functions take `&PgPool` and return `Result<T, KoraError>`.

**API → Schema:** Handlers call `SchemaHandler` trait methods for parsing/validation before passing to storage.

**API → Cache:** Handlers check cache before storage on reads. Storage writes invalidate cache entries.

**Data Flow (write path):**
```
HTTP request → axum handler → SchemaHandler::parse() → storage::insert() → cache::put() → HTTP response
```

**Data Flow (read path):**
```
HTTP request → axum handler → cache::get() → [miss?] → storage::get() → cache::put() → HTTP response
```

## Architecture Validation Results

### Coherence Validation ✅

**Decision Compatibility:**
All crates are tokio-rs ecosystem (axum, sqlx, prost, tracing) — shared runtime, patterns, and release cadence. DashMap is sync but operations complete in nanoseconds without crossing await points — optimal for this cache pattern (faster than async locks for sub-microsecond operations). figment + serde for config aligns with serde usage everywhere. thiserror + axum IntoResponse creates a clean error pipeline.

**Pattern Consistency:**
Naming conventions are coherent across all layers: snake_case DB, PascalCase Rust, Confluent-exact API paths. Error flow is unified: `Result<T, KoraError>` everywhere with centralized Confluent error code mapping. Structure patterns align with trait dispatch (`SchemaHandler` → 3 implementations via enum). TDD + clippy pedantic + `deny(missing_docs)` — enforcement rules are mutually coherent and non-contradictory.

**Structure Alignment:**
`api/` has one file per resource mapping 1:1 with Confluent endpoints. `storage/` is separate from `api/` — handlers call storage directly with no unnecessary service layer. `schema/` is isolated with trait dispatch — the 3 formats don't impact API or storage layers. Tests follow standard Rust: unit tests co-located, integration tests in `tests/`.

### Requirements Coverage Validation ✅

**Functional Requirements — 49/49 covered:**

| FR Category | FRs | Primary Files | Status |
|---|---|---|---|
| Schema Management (14 FRs) | Register, retrieve, list, soft/hard delete, list types, subjects/versions by ID | `api/subjects.rs`, `api/schemas.rs`, `storage/schemas.rs`, `storage/subjects.rs` | ✅ |
| Schema Formats (5 FRs) | Avro, JSON Schema, Protobuf parsing + reference resolution | `schema/avro.rs`, `schema/json_schema.rs`, `schema/protobuf.rs`, `storage/references.rs` | ✅ |
| Compatibility (13 FRs) | 7 modes, config CRUD, compatibility testing | `api/compatibility.rs`, `api/config_endpoints.rs`, `schema/handler.rs`, `storage/config.rs` | ✅ |
| Additional API Coverage (6 FRs) | List all schemas, raw schema text, referencedby, verbose compat, compat against all versions | `api/schemas.rs`, `api/subjects.rs`, `api/compatibility.rs` | ✅ |
| Storage (4 FRs) | PG storage, auto-migrations, embedded/external PG | `storage/mod.rs`, `migrations/`, `config.rs`, `Dockerfile` | ✅ |
| Registry Mode (2 FRs) | Get/set mode | `api/mode.rs`, `storage/config.rs` | ✅ |
| Observability (2 FRs) | Prometheus metrics, health check | `metrics.rs`, `api/health.rs` | ✅ |
| API Conformance (3 FRs) | Error codes, content-type, sequential IDs | `error.rs`, `api/mod.rs`, `storage/schemas.rs` | ✅ |

**Non-Functional Requirements — All addressed:**

| NFR | Architectural Support | Status |
|---|---|---|
| P99 < 5ms reads | DashMap in-process cache, hot path schema-by-ID | ✅ |
| P99 < 20ms writes | Direct sqlx → PG, no intermediate layers | ✅ |
| P99 < 100ms compatibility | Enum dispatch (not dyn), in-memory operations | ✅ |
| 10K+ schemas | PG + index strategy (`idx_{table}_{columns}`) | ✅ |
| Zero data loss | PG ACID, atomic operations, no in-memory-only state | ✅ |
| Graceful shutdown | tokio signal handler + connection draining | ✅ |
| Confluent wire-compat | Error codes, content-type, exact paths, response shapes | ✅ |
| Prometheus metrics | `metrics` + `metrics-exporter-prometheus` | ✅ |
| < 100MB RSS | No heavy framework, no runtime reflection | ✅ |
| < 3s cold start | Rust binary, no JVM, no interpreter | ✅ |

### Implementation Readiness Validation ✅

**Decision Completeness:**
All crates specified with exact versions. `cargo add` commands ready to copy-paste. Implementation sequence ordered in 8 logical steps. Code Philosophy in 6 clear, enforceable rules.

**Structure Completeness:**
Every file listed with its purpose. Data flow boundaries documented (write path / read path). FR-to-file mapping complete for all 9 categories.

**Pattern Completeness:**
Naming covers DB, API, Rust code — no blind spots. Error flow documented step by step. Logging levels defined with structured examples. Validation boundary clear: "validate at API, storage trusts input".

### Gap Analysis Results

**Critical Gaps: None.**

**Minor Documentation Gaps (non-blocking):**

1. Schema deletion protection (cross-cutting concern #5) — dependency graph query will be a simple SQL `EXISTS` check at implementation time. No additional architectural decision needed.
2. `SchemaHandler` trait signature not detailed — intentionally deferred to implementation stories. The trait is straightforward: `parse`, `canonical_form`, `check_compatibility`, `validate_references`.

### Architecture Completeness Checklist

**✅ Requirements Analysis**
- [x] Project context thoroughly analyzed
- [x] Scale and complexity assessed
- [x] Technical constraints identified
- [x] Cross-cutting concerns mapped

**✅ Architectural Decisions**
- [x] Critical decisions documented with versions
- [x] Technology stack fully specified
- [x] Integration patterns defined
- [x] Performance considerations addressed

**✅ Implementation Patterns**
- [x] Naming conventions established
- [x] Structure patterns defined
- [x] Communication patterns specified
- [x] Process patterns documented

**✅ Project Structure**
- [x] Complete directory structure defined
- [x] Component boundaries established
- [x] Integration points mapped
- [x] Requirements to structure mapping complete

### Architecture Readiness Assessment

**Overall Status:** READY FOR IMPLEMENTATION

**Confidence Level:** HIGH — all 49 FRs and 16 NFRs have explicit architectural support, zero critical gaps, coherent technology stack.

**Key Strengths:**
- 100% tokio-rs ecosystem coherence
- Zero unnecessary abstraction — handlers → storage direct
- TDD enforced systematically as non-negotiable rule
- Confluent wire-compatibility as primary architectural driver
- Logical, incremental implementation sequence

**AI Agent Implementation Guidelines:**
- Follow all architectural decisions exactly as documented
- Use implementation patterns consistently across all components
- Respect project structure and boundaries
- TDD: test first, code second, refactor third — no exceptions
- Refer to this document for all architectural questions

**First Implementation Priority:** Project scaffold with `cargo init` + crate dependencies + DB migrations + health check endpoint
