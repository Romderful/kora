---
stepsCompleted: ["step-01-init", "step-02-discovery", "step-02b-vision", "step-02c-executive-summary", "step-03-success", "step-04-journeys", "step-05-domain-skipped", "step-06-innovation-skipped", "step-07-project-type", "step-08-scoping-skipped", "step-09-functional", "step-10-nonfunctional", "step-11-polish"]
inputDocuments: ["_bmad-output/planning-artifacts/product-brief-kora.md"]
workflowType: 'prd'
documentCounts:
  briefs: 1
  research: 0
  projectDocs: 0
  projectContext: 0
classification:
  projectType: api_backend
  domain: general
  complexity: low
  projectContext: greenfield
---

# Product Requirements Document - Kora

**Author:** Mindsky
**Date:** 2026-04-04

## Executive Summary

Kora is a schema registry built in Rust with PostgreSQL storage, implementing the full Confluent Schema Registry REST API. It serves data engineers and platform teams who need a lightweight, high-performance registry without the overhead of JVM runtimes or Python environments.

Existing schema registries store data in Kafka topics — creating a circular dependency where the registry depends on the very system it validates. They are built in languages (Java, Python) that introduce runtime overhead, garbage collection pauses, and operational complexity. Under production load with thousands of schemas and concurrent connections, latency degrades significantly.

Kora eliminates these problems. A single compiled binary with sub-millisecond schema lookups, backed by a database every team already runs. It supports Avro, JSON Schema, and Protobuf, all seven compatibility modes, and adds schema comparison — the ability to programmatically diff two schema versions and understand exactly what changed.

### What Makes This Special

**Performance as a feature.** Kora is compiled native code. No JVM warmup, no Python interpreter, no garbage collection. Schema lookups in microseconds, registration throughput that scales linearly under concurrent load. The performance gap between Kora and existing registries is not incremental — it is an order of magnitude.

**Schemas in a real database.** PostgreSQL storage means SQL-queryable schema catalogs, standard backup/restore with pg_dump, no circular Kafka dependency, and integration with tooling teams already operate. Schemas become first-class database citizens.

**Schema comparison.** No existing registry offers a programmatic diff API. Kora does: field additions, removals, type changes, compatibility impact — all accessible via API. This enables schema-aware CI/CD gates and clear visibility into schema evolution.

## Project Classification

- **Type:** API backend — REST service consumed by Kafka tooling (serializers, connectors, CLI)
- **Domain:** Data infrastructure (general — no regulatory constraints)
- **Complexity:** Low domain complexity, high technical complexity (schema parsing, compatibility algorithms, Rust)
- **Context:** Greenfield — new project, no existing codebase

## Success Criteria

### User Success

- A data engineer installs Kora (`docker run` or single binary), points their existing Kafka tooling at it, and everything works — zero configuration changes, zero code changes
- Schema registration, lookup, and compatibility checking behave identically to the Confluent Schema Registry API — verified against the Confluent compatibility test suite
- Schema comparison delivers typed, semantic diffs (not text diffs) that are immediately actionable: each change has a path, a type, and a breaking/compatible verdict
- The operational experience is minimal: one binary, one PostgreSQL connection, standard `pg_dump` backups — no JVM tuning, no Kafka topic management, no Python environment

### Technical Success

- Order-of-magnitude performance improvement over Python-based registries on both lookups and registration throughput (specific targets in Non-Functional Requirements)
- All seven compatibility modes correctly enforced for Avro, JSON Schema, and Protobuf
- Schema comparison API returns structured diffs with typed change classifications per format (Avro: `FIELD_ADDED`, `TYPE_WIDENED`, etc. — JSON Schema: `PROPERTY_ADDED`, `REQUIRED_REMOVED`, etc. — Protobuf: `FIELD_ADDED`, `FIELD_NUMBER_CHANGED`, etc.)
- PostgreSQL storage with proper schema design — queryable catalog, standard backup/restore, no Kafka dependency

### Quality Bar

This project ships at an extreme quality bar. Every endpoint, every compatibility rule, every schema format is thoroughly tested. Correctness is non-negotiable — a schema registry that returns wrong compatibility results is worse than no registry at all.

## Product Scope

### MVP

- Full Confluent Schema Registry REST API (all endpoints: subjects, versions, schemas by ID, compatibility, config)
- Avro, JSON Schema, and Protobuf format support
- All compatibility modes (BACKWARD, FORWARD, FULL, NONE + TRANSITIVE variants)
- PostgreSQL storage backend
- Schema comparison API: pairwise diff between two versions (`GET /subjects/{subject}/versions/{v1}/diff/{v2}`) and arbitrary schema diff (`POST /schemas/diff`)
- Chain diff: cumulative diff across version history (v1 → vN)
- Structured semantic diff response: path, change type, from/to values, breaking verdict, summary
- Prometheus metrics endpoint
- Docker all-in-one image (embedded PostgreSQL, no external dependency required)
- Single binary distribution
- Health check endpoints

## User Journeys

### Journey 1: Ravi — Data Engineer, Transparent Migration

Ravi manages 200+ Debezium connectors on a Kafka cluster. His current registry slows under load — lookups exceed 50ms at peak, and consumers sporadically timeout. His manager asks him to fix it.

He discovers Kora. `docker run -p 8081:8081 kora` — that's it. No `DATABASE_URL`, no PG to provision. The image ships with its own internal PostgreSQL. He changes one environment variable in his Kafka Connect deployment: the schema registry URL. Redeploys. Nothing breaks. His connectors keep running. He checks his dashboards — lookups at 0.2ms. He never touches the registry again.

Six months later, the team wants high availability — two Kora instances behind a load balancer. Ravi externalizes PG: adds `DATABASE_URL` pointing to their RDS, both instances share the same data. The image detects the variable and stops starting the internal PG. Clean migration, zero downtime.

**What this journey reveals:** perfect API compatibility, zero-downtime migration, immediate performance gain, Docker all-in-one with embedded PG, transparent transition to external PG for HA.

### Journey 2: Nadia — Platform Engineer, Day-2 Operations

Nadia is ops. She deployed Kora 3 weeks ago — pointed at the team's production PG with a dedicated schema. Today, a developer registered a schema that breaks backward compatibility on a critical subject. Kora rejects the registration with a 409 and a clear message explaining the conflict. The dev fixes it and retries.

Friday, weekly backup: `pg_dump kora_db > backup.sql`. No special procedure, no Kafka snapshots. She checks `/metrics` in Grafana — 1,200 schemas, 45 subjects, P99 at 0.3ms, zero errors. She configures `FULL_TRANSITIVE` compatibility mode on production subjects, `NONE` on dev subjects.

**What this journey reveals:** clear error messages on compatibility rejections, standard PG backup, Prometheus metrics, per-subject configuration, co-location in an existing PG.

### Journey 3: Marcus — Platform Builder, Embedded Integration

Marcus builds a streaming platform. He needs an embedded schema registry — not an external dependency with its own Kafka cluster. He evaluates Kora: a 15MB binary, the standard Confluent API, and two deployment modes — all-in-one Docker for small clients, `DATABASE_URL` to the platform's PG for larger ones.

He integrates Kora into his Helm chart. For single-tenant deployments, he uses the all-in-one image. For multi-tenant, he points to his shared PG with a dedicated schema per tenant. His clients use standard Confluent serializers — they don't even know they're talking to Kora.

He uses the schema comparison API to build a feature in his UI: display the diff between versions when a user updates a schema. `GET /subjects/orders-value/versions/2/diff/5` returns structured JSON with each change typed. He renders it in his frontend in 2 hours.

**What this journey reveals:** deployment flexibility (all-in-one vs external PG), lightweight distribution, programmatically consumable comparison API, total client-side transparency.

### Journey Requirements Summary

| Capability | Revealed By |
|---|---|
| Full Confluent API compatibility | Ravi (migration), Marcus (client transparency) |
| Sub-millisecond performance | Ravi (latency fix), Nadia (monitoring) |
| Docker all-in-one with embedded PG | Ravi (quick start), Marcus (small deployments) |
| External PG via DATABASE_URL | Ravi (HA), Nadia (ops), Marcus (multi-tenant) |
| PostgreSQL backup (pg_dump) | Nadia (ops) |
| Compatibility mode enforcement | Nadia (rejection), Ravi (safety) |
| Per-subject configuration | Nadia (dev vs prod) |
| Clear error messages on rejection | Nadia (dev troubleshooting) |
| Prometheus metrics endpoint | Nadia (monitoring) |
| Schema comparison API | Marcus (UI integration) |
| Single binary / Docker distribution | Ravi (install), Marcus (embed) |
| Health check endpoints | Nadia (ops) |

## API Backend Requirements

### Endpoint Specification

Kora implements the complete Confluent Schema Registry REST API. All endpoints follow the exact Confluent path structure — no version prefix, no namespace modification. Drop-in compatible with all Confluent serializers and tooling.

**Confluent-Compatible Endpoints:**
- `GET /subjects` — list all subjects
- `GET /subjects/{subject}/versions` — list versions for a subject
- `GET /subjects/{subject}/versions/{version}` — get schema by subject and version
- `GET /subjects/{subject}/versions/latest` — get latest schema for a subject
- `POST /subjects/{subject}/versions` — register a new schema
- `POST /subjects/{subject}` — check if a schema is registered
- `DELETE /subjects/{subject}` — soft-delete a subject
- `DELETE /subjects/{subject}?permanent=true` — hard-delete a subject
- `DELETE /subjects/{subject}/versions/{version}` — soft-delete a schema version
- `DELETE /subjects/{subject}/versions/{version}?permanent=true` — hard-delete a schema version
- `GET /schemas/ids/{id}` — get schema by global ID
- `GET /schemas/ids/{id}/subjects` — list subjects associated with a schema ID
- `GET /schemas/ids/{id}/versions` — list versions associated with a schema ID
- `GET /schemas/types` — list supported schema types
- `POST /compatibility/subjects/{subject}/versions/{version}` — test compatibility
- `GET /config` — get global compatibility config
- `PUT /config` — update global compatibility config
- `GET /config/{subject}` — get per-subject compatibility config
- `PUT /config/{subject}` — update per-subject compatibility config
- `DELETE /config/{subject}` — delete per-subject config (fall back to global)
- `GET /mode` — get registry mode (READWRITE, READONLY, IMPORT)
- `PUT /mode` — set registry mode

**Kora Extension Endpoints:**
- `GET /subjects/{subject}/versions/{v1}/diff/{v2}` — pairwise semantic diff between two versions
- `POST /schemas/diff` — arbitrary schema diff (schemas provided in body)
- `GET /subjects/{subject}/versions/{v1}/diff/{v2}/chain` — cumulative chain diff across version range
- `GET /metrics` — Prometheus metrics
- `GET /health` — health check

### Authentication Model

No authentication in MVP. Kora is designed for deployment behind private networks, VPNs, or service meshes — consistent with how schema registries are typically operated in production.

### Data Formats

- **Request/Response:** JSON (`application/vnd.schemaregistry.v1+json`)
- **Schema formats:** Avro, JSON Schema, Protobuf (schema content transmitted as JSON-encoded strings)
- **Error responses:** Confluent-compatible error format (`{"error_code": 42201, "message": "..."}`)

### Error Codes

Confluent-compatible error codes:
- `40401` — Subject not found
- `40402` — Version not found
- `40403` — Schema not found
- `42201` — Invalid schema
- `42202` — Invalid version
- `409` — Incompatible schema (compatibility check failed)
- `50001` — Backend store error
- `50003` — Forwarding error

### API Documentation

OpenAPI/Swagger specification generated from the Confluent Schema Registry API spec, extended with Kora-specific diff endpoints. Serves as both documentation and conformance reference.

## Functional Requirements

### Schema Management

- FR1: API consumer can register a new schema under a subject
- FR2: API consumer can retrieve a schema by its global ID
- FR3: API consumer can retrieve a schema by subject and version number
- FR4: API consumer can retrieve the latest schema for a subject
- FR5: API consumer can check if a specific schema is already registered under a subject
- FR6: API consumer can list all registered subjects
- FR7: API consumer can list all versions of a subject
- FR8: API consumer can soft-delete a subject
- FR9: API consumer can soft-delete a specific version of a subject
- FR10: API consumer can permanently delete a subject (hard delete)
- FR11: API consumer can permanently delete a specific version (hard delete)
- FR12: API consumer can list supported schema types
- FR13: API consumer can retrieve the list of subjects associated with a schema ID
- FR14: API consumer can retrieve the list of versions associated with a schema ID

### Schema Formats

- FR15: System can parse, validate, and store Avro schemas
- FR16: System can parse, validate, and store JSON Schema schemas
- FR17: System can parse, validate, and store Protobuf schemas
- FR18: System can resolve and store schema references (Protobuf imports, JSON Schema `$ref`)
- FR19: System validates referenced schemas exist before accepting registration

### Compatibility

- FR20: API consumer can test compatibility of a schema against a specific version of a subject
- FR21: API consumer can get the global compatibility configuration
- FR22: API consumer can update the global compatibility configuration
- FR23: API consumer can get the per-subject compatibility configuration
- FR24: API consumer can update the per-subject compatibility configuration
- FR25: API consumer can delete a per-subject configuration (falling back to global)
- FR26: System enforces BACKWARD compatibility mode
- FR27: System enforces FORWARD compatibility mode
- FR28: System enforces FULL compatibility mode
- FR29: System enforces NONE compatibility mode
- FR30: System enforces BACKWARD_TRANSITIVE compatibility mode
- FR31: System enforces FORWARD_TRANSITIVE compatibility mode
- FR32: System enforces FULL_TRANSITIVE compatibility mode

### Schema Comparison

- FR33: API consumer can get a semantic diff between two versions of a subject
- FR34: API consumer can submit two arbitrary schemas and get a semantic diff
- FR35: API consumer can get a cumulative chain diff across a range of versions
- FR36: System returns typed change classifications per format (field added, type changed, etc.)
- FR37: System reports whether each change is breaking or compatible
- FR38: System provides a summary with total changes, breaking count, and compatible count

### Storage

- FR39: System stores all schemas and metadata in PostgreSQL
- FR40: System runs database migrations automatically on startup
- FR41: System connects to an external PostgreSQL when DATABASE_URL is provided
- FR42: Docker image starts an embedded PostgreSQL when no DATABASE_URL is provided

### Registry Mode

- FR43: Operator can get the current registry mode (READWRITE, READONLY, IMPORT)
- FR44: Operator can set the registry mode

### Observability

- FR45: Operator can access Prometheus metrics via `/metrics`
- FR46: Operator can check service health via `/health`

### API Conformance

- FR47: System returns Confluent-compatible error codes and error format
- FR48: System accepts and returns `application/vnd.schemaregistry.v1+json` content type
- FR49: System assigns globally unique sequential IDs to registered schemas

## Non-Functional Requirements

### Performance

- Schema lookups (GET by ID, by subject/version): P99 latency < 1ms under sustained load
- Schema registration: P99 latency < 10ms (includes parsing, validation, compatibility check, PG write)
- Schema diff (pairwise): P99 latency < 50ms for schemas up to 500 fields
- Concurrent connections: support 1,000+ simultaneous connections without degradation
- Startup time: cold start to serving requests < 3 seconds (excluding embedded PG boot)

### Reliability

- Data durability: all registered schemas persisted in PostgreSQL with ACID guarantees
- Crash recovery: restart and resume serving from PG state with zero data loss
- Graceful shutdown: complete in-flight requests before terminating
- No single point of data loss: PG backup/restore is the recovery path

### Scalability

- Support 100,000+ schema versions across 10,000+ subjects without performance degradation
- Memory footprint: < 100MB RSS for the Kora process under typical load (excluding PG)
- Linear throughput scaling with connection count up to system resource limits

### Integration

- 100% wire-compatible with Confluent Schema Registry API
- Compatible with: confluent-kafka-python, confluent-kafka-go, io.confluent serde (Java), Debezium, Kafka Connect, ksqlDB
- Content-type negotiation: `application/vnd.schemaregistry.v1+json` and `application/json`
- Confluent error code compatibility for all error responses
