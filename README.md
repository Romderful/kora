<div align="center">

# Kora

**A Confluent-compatible Schema Registry, built in Rust.**

PostgreSQL storage · Single binary · Sub-millisecond lookups · Zero JVM overhead

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![ghcr.io](https://img.shields.io/badge/ghcr.io-romderful%2Fkora-blue?logo=docker)](https://github.com/Romderful/Kora/pkgs/container/kora)

</div>

---

## Why Kora?

| | Confluent | Karapace | Kora |
|---|---|---|---|
| **Storage** | Kafka topic | Kafka topic | PostgreSQL |
| **Runtime** | JVM | Python | Native (Rust) |
| **Kafka dependency** | Required | Required | None |
| **API compatibility** | Reference | Partial | 100% wire-compatible |

**Confluent Schema Registry** stores schemas in a Kafka topic — a circular dependency where the tool validating your Kafka data depends on Kafka itself. It runs on the JVM with GC pauses, warmup time, and tuning overhead.

**Karapace** (Aiven) solves the licensing issue but introduces Python runtime overhead and still depends on Kafka.

**Kora** stores schemas as regular PostgreSQL rows. Back up with `pg_dump`, query with SQL, integrate with your existing tooling. Every endpoint, query parameter, and error code matches the Confluent API — existing serializers, connectors, and CLI tools work without modification.

---

## Quick Start

```bash
docker run -p 8080:8080 -e DATABASE_URL="postgres://user:pass@host:5432/kora" ghcr.io/romderful/kora:latest
```

```bash
curl http://localhost:8080/health
# {"status":"UP"}
```

---

## Image

A multi-arch image (linux/amd64 + linux/arm64) is published to GitHub Container Registry:

| Image | Tag | Size |
|---|---|---|
| `ghcr.io/romderful/kora` | `latest`, `v{version}` | ~24 MB |

The image contains a static Rust binary, tini, and SQL migrations. It requires an external PostgreSQL instance via `DATABASE_URL`.

---

## Configuration

All configuration is done via environment variables:

| Variable | Default | Description |
|---|---|---|
| `DATABASE_URL` | — (required) | PostgreSQL connection string |
| `HOST` | `0.0.0.0` | Server bind address |
| `PORT` | `8080` | Server listen port |
| `MAX_BODY_SIZE` | `16777216` | Maximum request body size (bytes, default 16 MB) |
| `DB_POOL_MAX` | `20` | Maximum database connections in the pool |
| `RUST_LOG` | `info` | Log level filter (`error`, `warn`, `info`, `debug`, `trace`) |

---

## API

Kora implements the full [Confluent Schema Registry REST API](https://docs.confluent.io/platform/current/schema-registry/develop/api.html). All standard clients (Kafka serializers, `kafka-schema-registry-maven-plugin`, `schema-registry-cli`, etc.) work without modification.

### Schemas

```
POST   /subjects/{subject}/versions                           Register a schema
GET    /subjects/{subject}/versions/{version}                 Get schema by version
GET    /subjects/{subject}/versions                           List versions
DELETE /subjects/{subject}/versions/{version}                 Delete version (soft/hard)
DELETE /subjects/{subject}                                    Delete subject (soft/hard)
POST   /subjects/{subject}                                    Check if schema is registered
GET    /schemas/ids/{id}                                      Get schema by global ID
GET    /schemas/ids/{id}/schema                               Get raw schema text
GET    /schemas/ids/{id}/subjects                             Subjects using this schema
GET    /schemas/ids/{id}/versions                             Versions using this schema
GET    /subjects                                              List all subjects
GET    /schemas                                               List all schemas
GET    /schemas/types                                         List supported types
GET    /subjects/{subject}/versions/{version}/schema          Raw schema text
GET    /subjects/{subject}/versions/{version}/referencedby    Referenced-by IDs
```

### Compatibility

```
POST   /compatibility/subjects/{subject}/versions/{version}   Test against version
POST   /compatibility/subjects/{subject}/versions             Test against all versions
GET    /config                                                Get global compatibility
PUT    /config                                                Set global compatibility
DELETE /config                                                Reset global compatibility
GET    /config/{subject}                                      Get subject compatibility
PUT    /config/{subject}                                      Set subject compatibility
DELETE /config/{subject}                                      Reset subject compatibility
```

### Registry Mode

```
GET    /mode                                                  Get global mode
PUT    /mode                                                  Set global mode
DELETE /mode                                                  Reset global mode
GET    /mode/{subject}                                        Get subject mode
PUT    /mode/{subject}                                        Set subject mode
DELETE /mode/{subject}                                        Reset subject mode
```

### Operations

```
GET    /health                                                Health check
GET    /metrics                                               Prometheus metrics
GET    /                                                      Root (Confluent compat)
POST   /                                                      Root (Confluent compat)
```

---

## Schema Formats

| Format | Registration | Compatibility | References |
|---|---|---|---|
| **Avro** | Canonical form normalization, Rabin fingerprinting | All 7 modes | Supported |
| **JSON Schema** | Validation, fingerprinting | All 7 modes | Supported |
| **Protobuf** | Parsing, validation, fingerprinting | All 7 modes | Supported |

---

## Compatibility Modes

Seven modes, configurable globally or per subject:

| Mode | Direction | Scope |
|---|---|---|
| `NONE` | Accept anything | — |
| `BACKWARD` | New schema can read old data | Last version |
| `BACKWARD_TRANSITIVE` | New schema can read old data | All versions |
| `FORWARD` | Old schema can read new data | Last version |
| `FORWARD_TRANSITIVE` | Old schema can read new data | All versions |
| `FULL` | Both directions | Last version |
| `FULL_TRANSITIVE` | Both directions | All versions |

Default: `BACKWARD` (matches Confluent).

---

## Deployment

### Helm

```bash
helm install kora oci://ghcr.io/romderful/kora/charts/kora \
  --set database.host=my-postgres.example.com \
  --set database.password=secret
```

Or with a full URL:

```bash
helm install kora oci://ghcr.io/romderful/kora/charts/kora \
  --set database.url="postgres://kora:secret@postgres:5432/kora?sslmode=require"
```

See [`chart/README.md`](chart/README.md) for all options.

### Docker Compose

```yaml
services:
  postgres:
    image: postgres:17-alpine
    environment:
      POSTGRES_DB: kora
      POSTGRES_USER: kora
      POSTGRES_PASSWORD: kora
    volumes:
      - pgdata:/var/lib/postgresql/data

  kora:
    image: ghcr.io/romderful/kora:latest
    depends_on:
      - postgres
    environment:
      DATABASE_URL: postgres://kora:kora@postgres:5432/kora
    ports:
      - "8080:8080"

volumes:
  pgdata:
```

### Docker

```bash
docker run -p 8080:8080 -e DATABASE_URL="postgres://user:pass@host:5432/kora" ghcr.io/romderful/kora:latest
```

---

## Development

Requires [just](https://github.com/casey/just), Rust, and Docker.

```
just dev       # Run locally with cargo (starts PG via Docker Compose)
just test      # Run all tests (starts PG automatically)
just fix       # Auto-fix formatting + clippy suggestions
just ci        # fmt + lint + test (same as CI)
just stop      # Stop all containers
just clean     # Remove containers, images, and volumes
```

### Build & Publish

```bash
just build               # Build + push image (multi-arch)
just build v0.4.0        # Tagged build
```

Override the registry: `KORA_IMAGE=my-registry.io/kora just build`

### All recipes

```
[build]    build                                    Build + push to ghcr.io (amd64 + arm64)
[dev]      dev, test                                Local development
[docker]   run, stop, clean                         Run images locally
[loadtest] smoke, load, stress, soak, contention    k6 load tests (requires k6)
[quality]  fmt, lint, fix, ci                       Code quality + CI entrypoint
```

---

## Architecture

```
Client → Kora (Axum + Tokio) → PostgreSQL
                ↓
         Prometheus /metrics
```

- **Axum** HTTP framework with Tower middleware
- **SQLx** compile-time verified queries with connection pooling
- **Rustls** TLS (no OpenSSL dependency)
- **Static musl binary** — single file, no runtime dependencies
- **tini** PID 1 in Docker for proper signal handling

---

## License

MIT
