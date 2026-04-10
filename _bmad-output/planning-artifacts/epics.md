---
stepsCompleted: [1, 2, 3, 4]
inputDocuments: ["_bmad-output/planning-artifacts/prd.md", "_bmad-output/planning-artifacts/architecture.md"]
---

# Kora - Epic Breakdown

## Overview

This document provides the complete epic and story breakdown for Kora, decomposing the requirements from the PRD and Architecture into implementable stories.

## Requirements Inventory

### Functional Requirements

FR1: API consumer can register a new schema under a subject
FR2: API consumer can retrieve a schema by its global ID
FR3: API consumer can retrieve a schema by subject and version number
FR4: API consumer can retrieve the latest schema for a subject
FR5: API consumer can check if a specific schema is already registered under a subject
FR6: API consumer can list all registered subjects
FR7: API consumer can list all versions of a subject
FR8: API consumer can soft-delete a subject
FR9: API consumer can soft-delete a specific version of a subject
FR10: API consumer can permanently delete a subject (hard delete)
FR11: API consumer can permanently delete a specific version (hard delete)
FR12: API consumer can list supported schema types
FR13: API consumer can retrieve the list of subjects associated with a schema ID
FR14: API consumer can retrieve the list of versions associated with a schema ID
FR15: System can parse, validate, and store Avro schemas
FR16: System can parse, validate, and store JSON Schema schemas
FR17: System can parse, validate, and store Protobuf schemas
FR18: System can resolve and store schema references (Protobuf imports, JSON Schema `$ref`)
FR19: System validates referenced schemas exist before accepting registration
FR20: API consumer can test compatibility of a schema against a specific version of a subject
FR21: API consumer can get the global compatibility configuration
FR22: API consumer can update the global compatibility configuration
FR23: API consumer can get the per-subject compatibility configuration
FR24: API consumer can update the per-subject compatibility configuration
FR25: API consumer can delete a per-subject configuration (falling back to global)
FR26: System enforces BACKWARD compatibility mode
FR27: System enforces FORWARD compatibility mode
FR28: System enforces FULL compatibility mode
FR29: System enforces NONE compatibility mode
FR30: System enforces BACKWARD_TRANSITIVE compatibility mode
FR31: System enforces FORWARD_TRANSITIVE compatibility mode
FR32: System enforces FULL_TRANSITIVE compatibility mode
FR33: API consumer can list all schemas with optional filters (subjectPrefix, deleted, latestOnly)
FR34: API consumer can get raw schema text by global ID (GET /schemas/ids/{id}/schema)
FR35: API consumer can get raw schema text by subject version (GET /subjects/{subject}/versions/{version}/schema)
FR36: API consumer can get the list of schema IDs that reference a given schema version (referencedby)
FR37: API consumer can test compatibility against all versions of a subject
FR38: Compatibility test endpoint supports verbose=true to return detailed incompatibility messages
FR39: System stores all schemas and metadata in PostgreSQL
FR40: System runs database migrations automatically on startup
FR41: System connects to an external PostgreSQL when DATABASE_URL is provided
FR42: Docker image starts an embedded PostgreSQL when no DATABASE_URL is provided
FR43: Operator can get the current registry mode (READWRITE, READONLY, READONLY_OVERRIDE, IMPORT)
FR44: Operator can set the registry mode
FR45: Operator can access Prometheus metrics via `/metrics`
FR46: Operator can check service health via `/health`
FR47: System returns Confluent-compatible error codes and error format
FR48: System accepts and returns `application/vnd.schemaregistry.v1+json` content type
FR49: System assigns globally unique sequential IDs to registered schemas

### NonFunctional Requirements

NFR1: Schema lookups (GET by ID, by subject/version): P99 latency < 1ms under sustained load
NFR2: Schema registration: P99 latency < 10ms (includes parsing, validation, compatibility check, PG write)
NFR3: Concurrent connections: support 1,000+ simultaneous connections without degradation
NFR4: Startup time: cold start to serving requests < 3 seconds (excluding embedded PG boot)
NFR5: Data durability: all registered schemas persisted in PostgreSQL with ACID guarantees
NFR6: Crash recovery: restart and resume serving from PG state with zero data loss
NFR7: Graceful shutdown: complete in-flight requests before terminating
NFR8: No single point of data loss: PG backup/restore is the recovery path
NFR9: Support 100,000+ schema versions across 10,000+ subjects without performance degradation
NFR10: Memory footprint: < 100MB RSS for the Kora process under typical load (excluding PG)
NFR11: Linear throughput scaling with connection count up to system resource limits
NFR12: 100% wire-compatible with Confluent Schema Registry API
NFR13: Compatible with: confluent-kafka-python, confluent-kafka-go, io.confluent serde (Java), Debezium, Kafka Connect, ksqlDB
NFR14: Content-type negotiation: `application/vnd.schemaregistry.v1+json` and `application/json`
NFR15: Confluent error code compatibility for all error responses

### Additional Requirements

- Greenfield project: `cargo init` + crate dependencies (no starter template)
- Docker all-in-one image with s6-overlay for process supervision (embedded PG + Kora)
- Single static binary distribution (`cargo build --release`)
- figment layered config: defaults → optional config file → env vars (`KORA_` prefix)
- TDD mandatory: Red-Green-Refactor, test first, no exceptions
- clippy pedantic: `#![deny(clippy::pedantic)]`
- `#![deny(missing_docs)]` at crate level
- Zero `unwrap()` outside `#[cfg(test)]`
- Automatic DB migrations on startup via sqlx
- Graceful shutdown via tokio signal handler + connection draining

### UX Design Requirements

N/A — Backend API project, no UI.

### FR Coverage Map

FR1: Epic 1 - Register a new schema under a subject
FR2: Epic 1 - Retrieve a schema by its global ID
FR3: Epic 1 - Retrieve a schema by subject and version number
FR4: Epic 1 - Retrieve the latest schema for a subject
FR5: Epic 1 - Check if a schema is already registered under a subject
FR6: Epic 1 - List all registered subjects
FR7: Epic 1 - List all versions of a subject
FR8: Epic 2 - Soft-delete a subject
FR9: Epic 2 - Soft-delete a specific version of a subject
FR10: Epic 2 - Permanently delete a subject (hard delete)
FR11: Epic 2 - Permanently delete a specific version (hard delete)
FR12: Epic 1 - List supported schema types
FR13: Epic 2 - Retrieve subjects associated with a schema ID
FR14: Epic 2 - Retrieve versions associated with a schema ID
FR15: Epic 1 - Parse, validate, and store Avro schemas
FR16: Epic 3 - Parse, validate, and store JSON Schema schemas
FR17: Epic 3 - Parse, validate, and store Protobuf schemas
FR18: Epic 2 - Resolve and store schema references
FR19: Epic 2 - Validate referenced schemas exist before registration
FR20: Epic 5 - Test compatibility against a specific version
FR21: Epic 4 - Get global compatibility configuration
FR22: Epic 4 - Update global compatibility configuration
FR23: Epic 4 - Get per-subject compatibility configuration
FR24: Epic 4 - Update per-subject compatibility configuration
FR25: Epic 4 - Delete per-subject configuration (fallback to global)
FR26: Epic 5 - Enforce BACKWARD compatibility mode
FR27: Epic 5 - Enforce FORWARD compatibility mode
FR28: Epic 5 - Enforce FULL compatibility mode
FR29: Epic 5 - Enforce NONE compatibility mode
FR30: Epic 5 - Enforce BACKWARD_TRANSITIVE compatibility mode
FR31: Epic 5 - Enforce FORWARD_TRANSITIVE compatibility mode
FR32: Epic 5 - Enforce FULL_TRANSITIVE compatibility mode
FR33: Epic 4 - List all schemas with filters
FR34: Epic 4 - Get raw schema text by global ID
FR35: Epic 4 - Get raw schema text by subject version
FR36: Epic 4 - Get schema IDs that reference a given version (referencedby)
FR37: Epic 5 - Test compatibility against all versions of a subject
FR38: Epic 5 - Compatibility verbose mode (detailed incompatibility messages)
FR39: Epic 1 - Store all schemas and metadata in PostgreSQL
FR40: Epic 1 - Run database migrations automatically on startup
FR41: Epic 1 - Connect to external PostgreSQL via DATABASE_URL
FR42: Epic 6 - Docker image with embedded PostgreSQL
FR43: Epic 6 - Get current registry mode
FR44: Epic 6 - Set registry mode
FR45: Epic 6 - Prometheus metrics via /metrics
FR46: Epic 1 - Health check via /health
FR47: Epic 4 - Confluent-compatible error codes and format
FR48: Epic 1 - Accept/return application/vnd.schemaregistry.v1+json
FR49: Epic 1 - Globally unique sequential schema IDs

## Epic List

### Epic 1: Core Schema Registry ✅ DONE
A developer can register Avro schemas, retrieve them by ID/subject/version, list subjects, and get a fully Confluent-compatible API experience.
Includes project scaffold, PostgreSQL storage, migrations, health check, error format, content-type negotiation, sequential ID allocation, and Avro handler.
**Stories:** 1.1–1.7 (all done)
**FRs covered:** FR1, FR2, FR3, FR4, FR5, FR6, FR7, FR12, FR15, FR39, FR40, FR41, FR46, FR48, FR49

### Epic 2: Schema Lifecycle Management ✅ DONE
A developer can manage the complete schema lifecycle: soft/hard delete subjects and versions, track which subjects/versions use a schema ID, and handle schema references with dependency protection.
**Stories:** 2.1–2.4 (all done)
**FRs covered:** FR8, FR9, FR10, FR11, FR13, FR14, FR18, FR19

### Epic 3: Multi-Format Support ✅ DONE
A developer can register JSON Schema and Protobuf schemas using the same workflow as Avro, with format-specific parsing, validation, and canonical form.
**Stories:** 3.1–3.2 (all done)
**FRs covered:** FR16, FR17

### Epic 4: Confluent API Parity
A developer gets 100% wire-compatibility with the Confluent Schema Registry: all error codes, query parameters, missing endpoints, and config behavior across all existing and new handlers.
**Stories:** 4.1 (done) → 4.2 → 4.3 → 4.4 → 4.5
**FRs covered:** FR21, FR22, FR23, FR24, FR25, FR33, FR34, FR35, FR36, FR47

### Epic 5: Compatibility Enforcement
A developer can test schema compatibility and the registry enforces all 7 compatibility modes on registration.
**Stories:** 5.1 → 5.2
**FRs covered:** FR20, FR26, FR27, FR28, FR29, FR30, FR31, FR32, FR37, FR38

### Epic 6: Operations & Packaging
An operator can control registry mode, access Prometheus metrics, and deploy via Docker all-in-one with embedded PostgreSQL and s6-overlay process supervision.
**Stories:** 6.1, 6.2, 6.3 (parallelizable)
**FRs covered:** FR42, FR43, FR44, FR45

## Epic 1: Core Schema Registry

A developer can register Avro schemas, retrieve them by ID/subject/version, list subjects, and get a fully Confluent-compatible API experience.

### Story 1.1: Project Scaffold, Database & Health Check

As a **developer**,
I want a running Kora server with PostgreSQL connectivity and a health endpoint,
So that I have the foundation to build all schema registry features on.

**Acceptance Criteria:**

**Given** a fresh checkout of the repository
**When** I run `cargo build`
**Then** the project compiles with zero warnings under `clippy::pedantic`
**And** `deny(missing_docs)` is enforced at crate level

**Given** a running PostgreSQL instance
**When** Kora starts with `DATABASE_URL` configured
**Then** database migrations run automatically on startup
**And** the `schemas`, `subjects`, `schema_references`, and `config` tables are created

**Given** a running Kora server
**When** I send `GET /health`
**Then** I receive HTTP 200 with PG connectivity confirmed
**And** if PG is unreachable, I receive HTTP 503

**Given** any API error
**When** the error response is returned
**Then** it follows Confluent format: `{"error_code": <int>, "message": "<string>"}`
**And** `Content-Type` is `application/vnd.schemaregistry.v1+json`

**Given** any request
**When** the request includes `Accept: application/vnd.schemaregistry.v1+json` or `application/json`
**Then** both content types are accepted (content-type negotiation)

**Note:** Confluent query parameter parity (pagination, normalize, deleted filters, subjectPrefix, etc.) deferred to Epic 4 stories 4.2/4.3. The `GET /` root endpoint and full set of 19 Confluent error codes are also deferred to story 4.2.

**FRs:** FR39, FR40, FR41, FR46, FR48

### Story 1.2: Register Avro Schema

As a **developer**,
I want to register an Avro schema under a subject,
So that my producers and consumers can serialize/deserialize data using a shared schema.

**Acceptance Criteria:**

**Given** a valid Avro schema JSON
**When** I send `POST /subjects/{subject}/versions` with `{"schema": "<avro_json>"}`
**Then** I receive HTTP 200 with `{"id": <globally_unique_sequential_id>}`
**And** the schema is stored in PostgreSQL with its canonical form

**Given** the same Avro schema is registered again under the same subject
**When** I send `POST /subjects/{subject}/versions`
**Then** I receive the existing schema ID (idempotent — no duplicate version created)

**Given** an invalid Avro schema
**When** I send `POST /subjects/{subject}/versions`
**Then** I receive HTTP 422 with Confluent error code 42201

**Given** a valid schema with `schemaType` omitted
**When** I send `POST /subjects/{subject}/versions`
**Then** the system defaults to Avro (Confluent default behavior)

**Note:** Confluent query parameter parity (pagination, normalize, deleted filters, subjectPrefix, etc.) deferred to Epic 4 stories 4.2/4.3.

**FRs:** FR1, FR15, FR49

### Story 1.3: Retrieve Schema by Global ID

As a **developer**,
I want to retrieve a schema by its global ID,
So that my deserializers can resolve schemas from the ID embedded in Kafka messages.

**Acceptance Criteria:**

**Given** a registered schema with ID 1
**When** I send `GET /schemas/ids/1`
**Then** I receive HTTP 200 with `{"schema": "<schema_json>", "schemaType": "AVRO", "references": [...]}`

**Given** a schema whose subject/version has been soft-deleted
**When** I send `GET /schemas/ids/1`
**Then** the schema is still returned (schemas are immutable by global ID — always retrievable regardless of subject soft-delete status)

**Given** a non-existent schema ID
**When** I send `GET /schemas/ids/999`
**Then** I receive HTTP 404 with Confluent error code 40403

**Note:** Confluent query parameter parity (pagination, normalize, deleted filters, subjectPrefix, etc.) deferred to Epic 4 stories 4.2/4.3. The `fetchMaxId`, `subject` query params and `schemaType` omission for AVRO are deferred to story 4.2.

**FRs:** FR2

### Story 1.4: Retrieve Schema by Subject and Version

As a **developer**,
I want to retrieve a schema by subject name and version number (or "latest"),
So that I can inspect specific versions or always get the most recent schema.

**Acceptance Criteria:**

**Given** subject "orders-value" with versions 1, 2, 3
**When** I send `GET /subjects/orders-value/versions/2`
**Then** I receive HTTP 200 with `{"subject": "orders-value", "id": <id>, "version": 2, "schema": "<json>", "schemaType": "AVRO"}`

**Given** subject "orders-value" with 3 versions
**When** I send `GET /subjects/orders-value/versions/latest`
**Then** I receive the version 3 schema

**Given** a non-existent subject
**When** I send `GET /subjects/unknown/versions/1`
**Then** I receive HTTP 404 with Confluent error code 40401

**Given** a valid subject but non-existent version
**When** I send `GET /subjects/orders-value/versions/99`
**Then** I receive HTTP 404 with Confluent error code 40402

**Note:** Confluent query parameter parity (pagination, normalize, deleted filters, subjectPrefix, etc.) deferred to Epic 4 stories 4.2/4.3. The `deleted=true` param on get-by-version and negative version indexing / 42202 error are deferred to stories 4.2/4.3.

**FRs:** FR3, FR4

### Story 1.5: List Subjects and Versions

As a **developer**,
I want to list all subjects and all versions of a subject,
So that I can discover available schemas in the registry.

**Acceptance Criteria:**

**Given** registered subjects "orders-value" and "users-value"
**When** I send `GET /subjects`
**Then** I receive HTTP 200 with `["orders-value", "users-value"]`

**Given** no registered subjects
**When** I send `GET /subjects`
**Then** I receive HTTP 200 with `[]`

**Given** subject "orders-value" with versions 1, 2, 3
**When** I send `GET /subjects/orders-value/versions`
**Then** I receive HTTP 200 with `[1, 2, 3]`

**Given** subject with soft-deleted versions
**When** I send `GET /subjects/orders-value/versions?deleted=true`
**Then** soft-deleted versions are included

**Note:** Confluent query parameter parity (pagination, normalize, deleted filters, subjectPrefix, etc.) deferred to Epic 4 stories 4.2/4.3. The `subjectPrefix`, `deletedOnly`, `deletedAsNegative`, and pagination (`offset`/`limit`) params are deferred to stories 4.2/4.3.

**FRs:** FR6, FR7

### Story 1.6: Check Schema Registration

As a **developer**,
I want to check if a schema is already registered under a subject,
So that I can verify whether my schema exists without registering a new version.

**Acceptance Criteria:**

**Given** subject "orders-value" has a registered schema
**When** I send `POST /subjects/orders-value` with `{"schema": "<matching_schema>"}`
**Then** I receive HTTP 200 with `{"subject": "orders-value", "id": <id>, "version": <ver>, "schema": "<json>"}`

**Given** a schema not registered under the subject
**When** I send `POST /subjects/orders-value` with `{"schema": "<unknown_schema>"}`
**Then** I receive HTTP 404 with Confluent error code 40403

**Note:** Confluent query parameter parity (pagination, normalize, deleted filters, subjectPrefix, etc.) deferred to Epic 4 stories 4.2/4.3. The `normalize=true` and `deleted=true` params on schema check are deferred to story 4.3.

**FRs:** FR5

### Story 1.7: List Supported Schema Types

As a **developer**,
I want to list the schema types the registry supports,
So that I know which formats I can use.

**Acceptance Criteria:**

**Given** a running Kora server
**When** I send `GET /schemas/types`
**Then** I receive HTTP 200 with `["AVRO", "JSON", "PROTOBUF"]`

**FRs:** FR12

## Epic 2: Schema Lifecycle Management

A developer can manage the complete schema lifecycle: soft/hard delete, reference tracking, and dependency protection.

### Story 2.1: Soft-Delete Subject and Versions

As a **developer**,
I want to soft-delete a subject or a specific version,
So that I can remove schemas from active use while preserving them for audit.

**Acceptance Criteria:**

**Given** subject "orders-value" with versions 1, 2, 3
**When** I send `DELETE /subjects/orders-value`
**Then** I receive HTTP 200 with `[1, 2, 3]` (list of soft-deleted versions)
**And** `GET /subjects` no longer includes "orders-value"
**And** `GET /subjects?deleted=true` includes "orders-value"

**Given** subject "orders-value" with versions 1, 2, 3
**When** I send `DELETE /subjects/orders-value/versions/2`
**Then** I receive HTTP 200 with `2`
**And** `GET /subjects/orders-value/versions` returns `[1, 3]`

**Given** a non-existent subject
**When** I send `DELETE /subjects/unknown`
**Then** I receive HTTP 404 with Confluent error code 40401

**FRs:** FR8, FR9

### Story 2.2: Hard-Delete Subject and Versions

As a **developer**,
I want to permanently delete a subject or version,
So that I can completely remove schemas that should not exist.

**Acceptance Criteria:**

**Given** a soft-deleted subject "orders-value"
**When** I send `DELETE /subjects/orders-value?permanent=true`
**Then** I receive HTTP 200 with the list of permanently deleted versions
**And** the schema data is removed from PostgreSQL

**Given** a soft-deleted version 2 of subject "orders-value"
**When** I send `DELETE /subjects/orders-value/versions/2?permanent=true`
**Then** I receive HTTP 200 with `2`

**Given** a subject that is NOT soft-deleted
**When** I send `DELETE /subjects/orders-value?permanent=true`
**Then** I receive HTTP 404 with Confluent error code 40401 (subject not found in soft-deleted state)

**Note:** Confluent query parameter parity (pagination, normalize, deleted filters, subjectPrefix, etc.) deferred to Epic 4 stories 4.2/4.3. The Confluent-specific 40405 error code (subject NOT soft-deleted) is deferred to story 4.2.

**FRs:** FR10, FR11

### Story 2.3: Schema ID Cross-References

As a **developer**,
I want to find all subjects and versions that use a given schema ID,
So that I can understand the impact of a schema across the registry.

**Acceptance Criteria:**

**Given** schema ID 1 is used by subjects "orders-value" (v1) and "users-value" (v2)
**When** I send `GET /schemas/ids/1/subjects`
**Then** I receive HTTP 200 with `["orders-value", "users-value"]`

**Given** schema ID 1 is registered as version 1 under "orders-value"
**When** I send `GET /schemas/ids/1/versions`
**Then** I receive HTTP 200 with `[{"subject": "orders-value", "version": 1}]`

**Given** a non-existent schema ID
**When** I send `GET /schemas/ids/999/subjects`
**Then** I receive HTTP 404 with Confluent error code 40403

**Note:** Confluent query parameter parity (pagination, normalize, deleted filters, subjectPrefix, etc.) deferred to Epic 4 stories 4.2/4.3. The `deleted=true` and pagination (`offset`/`limit`) params on cross-reference endpoints are deferred to stories 4.2/4.3.

**FRs:** FR13, FR14

### Story 2.4: Schema References and Dependency Protection

As a **developer**,
I want schemas to reference other schemas and be protected from deletion when referenced,
So that dependent schemas remain valid.

**Acceptance Criteria:**

**Given** a schema with `"references": [{"name": "User", "subject": "users-value", "version": 1}]`
**When** I send `POST /subjects/orders-value/versions` with the referencing schema
**Then** the system validates that "users-value" version 1 exists
**And** stores the reference relationship

**Given** a schema registration with a reference to a non-existent subject/version
**When** I send `POST /subjects/orders-value/versions`
**Then** I receive HTTP 422 with an error indicating the referenced schema was not found

**Given** schema "users-value" v1 is referenced by "orders-value" v1
**When** I attempt to hard-delete "users-value" v1
**Then** I receive HTTP 422 indicating the schema is referenced and cannot be deleted

**FRs:** FR18, FR19

## Epic 3: Multi-Format Support

A developer can register JSON Schema and Protobuf schemas with the same workflow as Avro.

### Story 3.1: JSON Schema Format Handler

As a **developer**,
I want to register JSON Schema schemas,
So that I can use JSON Schema for data validation in my pipeline.

**Acceptance Criteria:**

**Given** a valid JSON Schema document
**When** I send `POST /subjects/{subject}/versions` with `{"schemaType": "JSON", "schema": "<json_schema>"}`
**Then** I receive HTTP 200 with the assigned schema ID
**And** the schema is parsed, validated, and stored with its canonical form

**Given** an invalid JSON Schema document
**When** I send `POST /subjects/{subject}/versions` with `{"schemaType": "JSON", "schema": "<invalid>"}`
**Then** I receive HTTP 422 with Confluent error code 42201

**Given** a registered JSON Schema
**When** I retrieve it via `GET /schemas/ids/{id}`
**Then** the response includes `"schemaType": "JSON"`

**FRs:** FR16

### Story 3.2: Protobuf Format Handler

As a **developer**,
I want to register Protobuf schemas,
So that I can use Protobuf for high-performance serialization in my pipeline.

**Acceptance Criteria:**

**Given** a valid `.proto` definition as a string
**When** I send `POST /subjects/{subject}/versions` with `{"schemaType": "PROTOBUF", "schema": "<proto_def>"}`
**Then** I receive HTTP 200 with the assigned schema ID
**And** the schema is parsed, validated, and stored with its canonical form

**Given** an invalid Protobuf definition
**When** I send `POST /subjects/{subject}/versions` with `{"schemaType": "PROTOBUF", "schema": "<invalid>"}`
**Then** I receive HTTP 422 with Confluent error code 42201

**Given** a registered Protobuf schema
**When** I retrieve it via `GET /schemas/ids/{id}`
**Then** the response includes `"schemaType": "PROTOBUF"`

**FRs:** FR17

## Epic 4: Confluent API Parity

A developer gets 100% wire-compatibility with the Confluent Schema Registry: all error codes, query parameters, missing endpoints, and config behavior.

### Story 4.1: Compatibility Configuration CRUD ✅ DONE

As a **developer**,
I want to get and set compatibility configuration at global and per-subject levels,
So that I can control how schema evolution is enforced.

**Acceptance Criteria:**

**Given** a running Kora server with default config
**When** I send `GET /config`
**Then** I receive HTTP 200 with `{"compatibilityLevel": "BACKWARD"}`

**Given** I want to change the global compatibility
**When** I send `PUT /config` with `{"compatibility": "FULL"}`
**Then** I receive HTTP 200 with `{"compatibility": "FULL"}`

**Given** subject "orders-value" exists
**When** I send `PUT /config/orders-value` with `{"compatibility": "NONE"}`
**Then** I receive HTTP 200 with `{"compatibility": "NONE"}`

**Given** subject "orders-value" has per-subject config
**When** I send `GET /config/orders-value`
**Then** I receive HTTP 200 with `{"compatibilityLevel": "NONE"}`

**Given** subject "orders-value" has per-subject config
**When** I send `DELETE /config/orders-value`
**Then** I receive HTTP 200 with the compatibility level (current implementation returns global fallback)

**FRs:** FR21, FR22, FR23, FR24, FR25

**Note:** Confluent query parameter parity deferred to Epic 4 stories 4.2/4.3. Specifically: `defaultToGlobal` query param, `DELETE /config` (global reset), error code 40408, and `DELETE /config/{subject}` returning the *previous* level instead of the fallback — all deferred to story 4.2.

### Story 4.2: Confluent API Parity — Infrastructure & Core Params

As a **developer using Confluent-compatible tooling**,
I want Kora's core infrastructure (error codes, pagination, root endpoint, config behavior) to match the Confluent Schema Registry,
so that client libraries work without modification.

**Acceptance Criteria:**

**Given** a running Kora server
**When** I send `GET /` or `POST /`
**Then** I receive HTTP 200 with `{}` (empty JSON object)

**Given** any API error
**Then** all 19 Confluent error codes are supported (40401–40409, 40901, 42201–42206, 50001–50003)

**Given** `DELETE /subjects/{subject}?permanent=true` and the subject is NOT soft-deleted
**Then** I receive HTTP 404 with error code 40405

**Given** any list endpoint with `offset` and `limit` query params
**Then** results are paginated (offset default=0, limit default=-1 meaning unlimited)

**Given** `GET /config/{subject}` without `defaultToGlobal` and no per-subject config exists
**Then** I receive HTTP 404 with error code 40408

**Given** `DELETE /config`
**Then** global config resets to BACKWARD

**Given** `GET /schemas/ids/{id}` for an AVRO schema
**Then** `schemaType` is omitted from the response

**Given** `GET /schemas/ids/{id}?fetchMaxId=true`
**Then** response includes `"maxId"` field

**Given** `GET /subjects/{subject}/versions/0` or any negative integer
**Then** I receive HTTP 422 with error code 42202 (only "latest" and positive integers accepted)

**FRs:** FR47

### Story 4.3: Confluent API Parity — List & Lookup Params

As a **developer using Confluent-compatible tooling**,
I want all list and lookup endpoints to support the full set of Confluent query parameters,
so that every client library query works identically against Kora.

**Depends on:** Story 4.2

**Acceptance Criteria:**

**Given** `GET /subjects?subjectPrefix=orders`
**Then** only subjects starting with "orders" are returned (default `:*:` = match all)

**Given** `GET /subjects?deletedOnly=true`
**Then** ONLY soft-deleted subjects are returned

**Given** `GET /subjects/{subject}/versions?deletedAsNegative=true`
**Then** soft-deleted versions appear as negative numbers

**Given** `POST /subjects/{subject}/versions?normalize=true`
**Then** the schema is normalized before fingerprint comparison and deduplication

**Given** `POST /subjects/{subject}?deleted=true`
**Then** soft-deleted schema matches are included in the lookup

**Given** `GET /schemas/ids/{id}/subjects?deleted=true`
**Then** soft-deleted subjects are included

**Given** `GET /subjects/{subject}/versions/2?deleted=true` for a soft-deleted version
**Then** the soft-deleted version is returned

**FRs:** FR47

### Story 4.4: List All Schemas and Raw Schema Text Endpoints

As a **developer**,
I want to list all schemas globally and retrieve raw schema text,
So that I can discover and access schemas using all Confluent-compatible endpoints.

**Depends on:** Story 4.2 (pagination, format params)

**Acceptance Criteria:**

**Given** registered schemas across multiple subjects
**When** I send `GET /schemas`
**Then** I receive HTTP 200 with a list of all schemas (subject, version, id, schemaType)

**Given** registered schemas and query parameter `subjectPrefix=orders`
**When** I send `GET /schemas?subjectPrefix=orders`
**Then** I receive only schemas whose subject starts with "orders"

**Given** query parameters `deleted=true`, `latestOnly=true`, `offset=10`, `limit=20`
**When** I send `GET /schemas?deleted=true&latestOnly=true&offset=10&limit=20`
**Then** results include soft-deleted schemas, only latest versions, paginated accordingly

**Given** a registered schema with ID 1
**When** I send `GET /schemas/ids/1/schema`
**Then** I receive HTTP 200 with the raw schema text only (no wrapper object)
**And** optional `subject` query param provides context for schema resolution

**Given** subject "orders-value" with version 2
**When** I send `GET /subjects/orders-value/versions/2/schema`
**Then** I receive HTTP 200 with the raw schema text only (no wrapper object)
**And** optional `deleted=true` includes soft-deleted versions

**FRs:** FR33, FR34, FR35

### Story 4.5: Referenced-By Lookup

As a **developer**,
I want to find which schemas reference a given schema version,
So that I can understand downstream dependencies before making changes.

**Depends on:** Story 4.2 (pagination, deleted param)

**Acceptance Criteria:**

**Given** schema "orders-value" v1 references "users-value" v1
**When** I send `GET /subjects/users-value/versions/1/referencedby`
**Then** I receive HTTP 200 with `[<schema_id_of_orders_v1>]`

**Given** a schema version with no dependents
**When** I send `GET /subjects/orders-value/versions/1/referencedby`
**Then** I receive HTTP 200 with `[]`

**Given** pagination on referencedby
**When** I send `GET /subjects/users-value/versions/1/referencedby?offset=0&limit=10`
**Then** results are paginated (limit=-1 means unlimited, default)

**Given** `deleted=true`
**When** I send `GET /subjects/users-value/versions/1/referencedby?deleted=true`
**Then** soft-deleted referencing schemas are included

**Given** a non-existent subject or version
**When** I send `GET /subjects/unknown/versions/1/referencedby`
**Then** I receive HTTP 404 with Confluent error code 40401

**FRs:** FR36

## Epic 5: Compatibility Enforcement

A developer can test schema compatibility and the registry enforces all 7 compatibility modes on registration.

### Story 5.1: Compatibility Test Endpoint

As a **developer**,
I want to test if a new schema is compatible with existing versions,
So that I can validate schema changes before registering them.

**Acceptance Criteria:**

**Given** subject "orders-value" with version 1 and compatibility mode BACKWARD
**When** I send `POST /compatibility/subjects/orders-value/versions/latest` with a backward-compatible schema
**Then** I receive HTTP 200 with `{"is_compatible": true}`

**Given** subject "orders-value" with compatibility mode BACKWARD
**When** I send `POST /compatibility/subjects/orders-value/versions/latest` with an incompatible schema
**Then** I receive HTTP 200 with `{"is_compatible": false}`

**Given** subject "orders-value" with compatibility mode BACKWARD and `verbose=true`
**When** I send `POST /compatibility/subjects/orders-value/versions/latest?verbose=true` with an incompatible schema
**Then** I receive HTTP 200 with `{"is_compatible": false, "messages": ["...descriptive incompatibility messages..."]}`

**Given** subject "orders-value" with versions 1, 2, 3
**When** I send `POST /compatibility/subjects/orders-value/versions` with a new schema
**Then** compatibility is tested against ALL versions (1, 2, 3)

**Given** `normalize=true` on compatibility check
**When** I send `POST /compatibility/subjects/orders-value/versions/latest?normalize=true`
**Then** the schema is normalized before compatibility testing

**Given** a non-existent subject
**When** I send `POST /compatibility/subjects/unknown/versions/latest`
**Then** I receive HTTP 404 with Confluent error code 40401

**FRs:** FR20, FR37, FR38

### Story 5.2: Enforce All Compatibility Modes (Avro)

As a **developer**,
I want the registry to enforce all 7 compatibility modes when registering schemas,
So that incompatible schema changes are rejected automatically.

**Acceptance Criteria:**

**Given** subject with BACKWARD mode and an existing Avro schema with a required field
**When** I register a new version that removes the required field
**Then** registration is rejected with HTTP 409 and Confluent error code 40901

**Given** subject with FORWARD mode
**When** I register a new version that adds a required field without default
**Then** registration is rejected with HTTP 409

**Given** subject with FULL mode
**When** I register a schema that is backward-compatible but not forward-compatible
**Then** registration is rejected

**Given** subject with NONE mode
**When** I register any valid schema
**Then** registration succeeds regardless of compatibility

**Given** subject with BACKWARD_TRANSITIVE mode and versions 1, 2, 3
**When** I register version 4
**Then** it must be backward-compatible with ALL previous versions (1, 2, 3), not just version 3

**Given** FORWARD_TRANSITIVE and FULL_TRANSITIVE modes
**When** registering a new version
**Then** the same transitive logic applies (checked against all versions)

**FRs:** FR26, FR27, FR28, FR29, FR30, FR31, FR32

## Epic 6: Operations & Packaging

An operator can control registry mode, access metrics, and deploy via Docker all-in-one.

### Story 6.1: Registry Mode Control

As an **operator**,
I want to get and set the registry mode (READWRITE, READONLY, READONLY_OVERRIDE, IMPORT),
So that I can control write access during maintenance or migration.

**Acceptance Criteria:**

**Given** a running Kora server
**When** I send `GET /mode`
**Then** I receive HTTP 200 with `{"mode": "READWRITE"}`

**Given** I want to set read-only mode
**When** I send `PUT /mode` with `{"mode": "READONLY"}`
**Then** I receive HTTP 200 with `{"mode": "READONLY"}`
**And** subsequent schema registration attempts return HTTP 422 with appropriate error

**Given** IMPORT mode is set
**When** I register a schema with an explicit ID
**Then** the system accepts the provided ID instead of auto-allocating

**Given** READONLY_OVERRIDE mode is set at subject level
**When** I attempt to register a schema under that subject
**Then** registration succeeds (READONLY_OVERRIDE allows writes despite global READONLY)

**Given** `force=true`
**When** I send `PUT /mode?force=true` with `{"mode": "READONLY"}`
**Then** the mode is set even if there are pending operations

**Given** subject "orders-value" exists
**When** I send `PUT /mode/orders-value` with `{"mode": "READONLY"}`
**Then** I receive HTTP 200 with `{"mode": "READONLY"}`

**Given** subject "orders-value" has per-subject mode
**When** I send `GET /mode/orders-value`
**Then** I receive the per-subject mode

**Given** subject has NO per-subject mode and `defaultToGlobal=true`
**When** I send `GET /mode/orders-value?defaultToGlobal=true`
**Then** I receive the global mode as fallback

**Given** subject has NO per-subject mode and `defaultToGlobal` is not set
**When** I send `GET /mode/orders-value`
**Then** I receive HTTP 404 with Confluent error code 40409 (subject mode not configured)

**Given** an invalid mode value
**When** I send `PUT /mode` with `{"mode": "INVALID"}`
**Then** I receive HTTP 422 with Confluent error code 42204 (invalid mode)

**Given** subject "orders-value" has per-subject mode
**When** I send `DELETE /mode/orders-value`
**Then** the per-subject mode is removed, falling back to global
**And** `recursive=true` propagates deletion to child subjects if applicable

**Given** a global mode override exists
**When** I send `DELETE /mode`
**Then** the global mode resets to READWRITE

**FRs:** FR43, FR44

### Story 6.2: Prometheus Metrics

As an **operator**,
I want to access Prometheus metrics,
So that I can monitor Kora's performance and health in my observability stack.

**Acceptance Criteria:**

**Given** a running Kora server
**When** I send `GET /metrics`
**Then** I receive HTTP 200 with Prometheus text exposition format
**And** metrics include: request count, request duration histogram, schema count, active connections

**FRs:** FR45

### Story 6.3: Docker All-in-One Packaging

As an **operator**,
I want a Docker image that runs Kora with embedded PostgreSQL,
So that I can deploy with zero external dependencies.

**Acceptance Criteria:**

**Given** the Docker image is built
**When** I run `docker run -p 8080:8080 kora`
**Then** embedded PostgreSQL starts automatically
**And** Kora starts and connects to the embedded PG
**And** `GET /health` returns HTTP 200

**Given** `DATABASE_URL` is set
**When** I run `docker run -e DATABASE_URL=... kora`
**Then** embedded PostgreSQL does NOT start
**And** Kora connects to the external PG

**Given** a running container
**When** I send `docker stop`
**Then** both Kora and PG shut down gracefully via s6-overlay

**FRs:** FR42
