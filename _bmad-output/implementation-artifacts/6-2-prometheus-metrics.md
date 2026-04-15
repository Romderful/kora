# Story 6.2: Prometheus Metrics

Status: done

## Story

As an **operator**,
I want to access Prometheus metrics via `GET /metrics`,
so that I can monitor Kora's performance and health in my observability stack.

## Acceptance Criteria

### AC1: Metrics Endpoint Returns Prometheus Text Format

**Given** a running Kora server
**When** I send `GET /metrics`
**Then** I receive HTTP 200 with `Content-Type: text/plain; version=0.0.4; charset=utf-8`
**And** the body is valid Prometheus text exposition format

### AC2: Request Count Metric

**Given** I have sent N requests to the server
**When** I query `GET /metrics`
**Then** I see `http_requests_total` counter with labels `method`, `path`, `status`
**And** the count reflects actual requests served

### AC3: Request Duration Histogram

**Given** I have sent requests to the server
**When** I query `GET /metrics`
**Then** I see `http_request_duration_seconds` histogram with labels `method`, `path`
**And** bucket boundaries cover typical latencies (1ms to 10s)

### AC4: Schema Count Gauge

**Given** schemas are registered in the registry
**When** I query `GET /metrics`
**Then** I see `kora_schema_count` gauge reflecting current schema content count

### AC5: Active Database Connections Gauge

**Given** a running Kora server
**When** I query `GET /metrics`
**Then** I see `kora_db_connections_in_use` gauge reflecting connections currently executing queries (`pool.size() - pool.num_idle()`)
**And** I see `kora_db_connections_idle` gauge reflecting idle connections in pool

## Tasks / Subtasks

- [x] Task 1: Add dependencies to `Cargo.toml` (AC: 1-5)
  - [x] 1.1: Add `metrics = "0.24"` to `[dependencies]`
  - [x] 1.2: Add `metrics-exporter-prometheus = "0.16"` to `[dependencies]`

- [x] Task 2: Initialize Prometheus recorder in `src/main.rs` (AC: 1)
  - [x] 2.1: Call `PrometheusBuilder::new().install_recorder()` after tracing init, before pool creation
  - [x] 2.2: Store the returned `PrometheusHandle` — passed to router via `AppState`
  - [x] 2.3: Pass the handle to `api::router()` via `AppState` struct

- [x] Task 3: Create metrics handler — `src/api/metrics.rs` (AC: 1)
  - [x] 3.1: Create `src/api/metrics.rs` with module doc
  - [x] 3.2: Add `pub mod metrics;` to `src/api/mod.rs`
  - [x] 3.3: Handler `get_metrics` extracts `State<PgPool>` and `State<PrometheusHandle>` via `FromRef`, returns `text/plain`

- [x] Task 4: Router changes — `src/api/mod.rs` (AC: 1-5)
  - [x] 4.1: Split router: API routes get content-type layer, `/metrics` route does not
  - [x] 4.2: Defined `AppState` with `#[derive(FromRef)]` — existing `State<PgPool>` extractors unchanged

- [x] Task 5: Request instrumentation middleware (AC: 2, 3)
  - [x] 5.1: Hand-rolled `track_metrics` async middleware in `src/api/middleware.rs`
  - [x] 5.2: Labels: `method`, `path` (via `MatchedPath`, "unmatched" for 404), `status`
  - [x] 5.3: Uses `metrics::counter!` and `metrics::histogram!` macros
  - [x] 5.4: Applied on merged router via `axum::middleware::from_fn`

- [x] Task 6: Business metrics — schema count gauge (AC: 4)
  - [x] 6.1: Used Option A — `SELECT COUNT(*) FROM schema_contents` at scrape time
  - [x] 6.2: Query runs in `get_metrics` handler using shared `PgPool`

- [x] Task 7: Database connection pool metrics (AC: 5)
  - [x] 7.1: `kora_db_connections_in_use` (size - idle) and `kora_db_connections_idle` gauges
  - [x] 7.2: Snapshotted at scrape time before `handle.render()`

- [x] Task 8: Integration tests (AC: 1-5)
  - [x] 8.1: Test `GET /metrics` returns HTTP 200
  - [x] 8.2: Test response content-type is `text/plain`
  - [x] 8.3: Test response body contains `http_requests_total` counter
  - [x] 8.4: Test response body contains `http_request_duration_seconds` histogram
  - [x] 8.5: Test response body contains `kora_schema_count`
  - [x] 8.6: Test response body contains `kora_db_connections` gauges
  - [x] 8.7: Test that after registering a schema, `kora_schema_count` increases
  - [x] 8.8: Test that `http_requests_total` counter increments after making requests

## Dev Notes

### Architecture-Specified Crates

The architecture document specifies:
- `metrics` (0.24.x) — lightweight metrics facade (counter!, histogram!, gauge! macros)
- `metrics-exporter-prometheus` (0.16.x) — Prometheus text exposition exporter

Do NOT use alternatives (opentelemetry, prometheus-client, etc.). These are the chosen crates.

### Content-Type Routing Problem

The current middleware uses `SetResponseHeaderLayer::overriding()` on the entire router — this forces `application/vnd.schemaregistry.v1+json` on ALL responses including `/metrics`. The `/metrics` endpoint MUST return `text/plain; version=0.0.4; charset=utf-8`.

**Solution pattern** — split the router:

```rust
pub fn router(pool: PgPool, handle: PrometheusHandle, max_body_size: usize) -> Router {
    // API routes get the Confluent content-type header
    let api = Router::new()
        .route("/", get(root).post(root))
        .route("/health", get(health::check_health))
        // ... all existing routes ...
        .layer(middleware::content_type_layer());

    // Operational routes do NOT get the Confluent content-type
    let ops = Router::new()
        .route("/metrics", get(metrics::get_metrics));

    api.merge(ops)
        .layer(DefaultBodyLimit::max(max_body_size))
        .with_state(AppState { pool, metrics_handle: handle })
}
```

This requires changing the shared state from bare `PgPool` to a struct that holds both pool and handle. Define in `src/api/mod.rs`:

```rust
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub metrics_handle: PrometheusHandle,
}
```

**Impact**: all existing handlers use `State(pool): State<PgPool>` — these must change to `State(state): State<AppState>` and access `state.pool`. This is a mechanical refactor across all handler files.

Alternatively, to minimize churn: use axum's `FromRef` derive so `State<PgPool>` continues to work:

```rust
#[derive(Clone, FromRef)]
pub struct AppState {
    pub pool: PgPool,
    pub metrics_handle: PrometheusHandle,
}
```

With `FromRef`, existing `State(pool): State<PgPool>` extractors keep working — no handler changes needed. The metrics handler uses `State(handle): State<PrometheusHandle>`. This is the preferred approach.

`FromRef` is in `axum::extract::FromRef` (re-exported from axum-core). Check that the axum version supports it (0.8.x does — it's a derive macro).

### Request Instrumentation Middleware

Create a hand-rolled axum middleware in `src/api/middleware.rs` (extend existing file). Do NOT use `tower-http::TraceLayer` — that's for tracing spans, not Prometheus counters/histograms. The middleware wraps each request:

1. Record start time
2. Extract method and matched route path
3. Call inner service
4. On response, record counter and histogram with labels

For the path label, use axum's `MatchedPath` extractor in the middleware to get the route template (e.g. `/subjects/{subject}/versions`) rather than the actual path (which would cause high cardinality).

**Important**: `MatchedPath` is available from the request extensions after routing. In axum 0.8, you can access it via `req.extensions().get::<MatchedPath>()`. If no matched path (404), use `"unmatched"`.

### Metrics Naming Convention

Follow Prometheus naming best practices:
- `http_requests_total` — counter (unit-less, `_total` suffix)
- `http_request_duration_seconds` — histogram (seconds unit in name)
- `kora_schema_count` — gauge (application-specific prefix)
- `kora_db_connections_in_use` — gauge
- `kora_db_connections_idle` — gauge

### PrometheusHandle Initialization

```rust
// In main.rs, after tracing init:
let metrics_handle = metrics_exporter_prometheus::PrometheusBuilder::new()
    .install_recorder()
    .expect("failed to install Prometheus recorder");
```

This sets the global recorder and returns a handle. The handle's `.render()` method produces the text exposition output.

### Scrape-Time DB Metrics

For schema count and pool stats, query at scrape time (inside the `/metrics` handler, before calling `handle.render()`). This is the simplest correct approach:

```rust
pub async fn get_metrics(
    State(pool): State<PgPool>,
    State(handle): State<PrometheusHandle>,
) -> Response {
    // Snapshot business metrics
    let schema_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM schema_contents")
        .fetch_one(&pool).await.unwrap_or(0);
    metrics::gauge!("kora_schema_count").set(schema_count as f64);

    // Snapshot pool metrics
    let size = pool.size() as f64;
    let idle = pool.num_idle() as f64;
    metrics::gauge!("kora_db_connections_in_use").set(size - idle);
    metrics::gauge!("kora_db_connections_idle").set(idle);

    // Render
    let body = handle.render();
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/plain; version=0.0.4; charset=utf-8")],
        body,
    ).into_response()
}
```

### Histogram Bucket Boundaries

The default buckets from `metrics-exporter-prometheus` are designed for HTTP latencies. If custom buckets are needed, configure via `PrometheusBuilder::set_buckets()`. The defaults (5ms, 10ms, 25ms, 50ms, 100ms, 250ms, 500ms, 1s, 2.5s, 5s, 10s) are reasonable for a schema registry.

### Testing Considerations

- The Prometheus recorder is **global** (process-wide singleton). `install_recorder()` **panics** if called twice. Use `PrometheusBuilder::new().try_install_recorder()` which returns `Err` instead of panicking — the test helper should use this so multiple test servers in the same process don't crash. Wrap with `std::sync::Once` or simply ignore the `Err` on second call (the already-installed recorder still works).
- Parse the response body as text and use `contains()` or regex to check for metric names — do NOT try to parse Prometheus format.
- The test server created by `spawn_server()` will need to be updated to install the recorder and pass the handle. The handle must be stored and shared (e.g., via a `static OnceLock<PrometheusHandle>`).

### Files to Create

| File | Purpose |
|---|---|
| `src/api/metrics.rs` | Metrics handler (`GET /metrics`) |

### Files to Modify

| File | Change |
|---|---|
| `Cargo.toml` | Add `metrics`, `metrics-exporter-prometheus` |
| `src/main.rs` | Install recorder, pass handle to router |
| `src/api/mod.rs` | Add `pub mod metrics`, define `AppState` with `FromRef`, split router for content-type |
| `src/api/middleware.rs` | Add request instrumentation middleware |
| `tests/common/mod.rs` | Update `spawn_server` to install recorder and pass handle |

### Files NOT to Modify

All existing handlers (`health.rs`, `schemas.rs`, `subjects.rs`, `compatibility.rs`, `mode.rs`) — `FromRef` keeps `State<PgPool>` working unchanged.

### Project Structure Notes

- `src/api/metrics.rs` follows the same pattern as `src/api/health.rs` — operational endpoint, simple handler
- The middleware extension in `src/api/middleware.rs` keeps all middleware in one place
- No new storage module needed — DB queries go directly in the handler

### Previous Story Intelligence

From story 6.1:
- Router registration: routes go in `src/api/mod.rs` — add `/metrics` route there
- Middleware layer order matters — layers applied last are closest to the handler
- Test pattern: `#[serial]` for tests that touch global state, always restore defaults
- Test server helper in `tests/common/mod.rs` — `spawn_server()` creates a fresh server + DB pool
- `file_serial` from `serial_test` crate used for cross-binary test isolation

### Git Intelligence

Recent commits follow one-commit-per-story pattern. The codebase has been through multiple Confluent wire-compatibility audits. This story is purely additive (new endpoint, new middleware) with minimal risk of regressions — the main risk is the content-type middleware split, which needs careful testing to ensure no existing endpoint loses its header.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 6.2] — Acceptance criteria
- [Source: _bmad-output/planning-artifacts/prd.md#FR45] — Prometheus metrics requirement
- [Source: _bmad-output/planning-artifacts/architecture.md#Observability Stack] — metrics + metrics-exporter-prometheus crates
- [Source: _bmad-output/planning-artifacts/architecture.md#Performance Requirements] — P99 targets for reads/writes/compat
- [Source: src/api/mod.rs] — Current router construction (lines 26-89)
- [Source: src/api/middleware.rs] — Content-type layer using overriding mode
- [Source: src/api/health.rs] — Handler pattern reference
- [Source: src/main.rs] — Server startup sequence
- [Source: src/config.rs] — KoraConfig structure
- [Source: Cargo.toml] — Current dependencies (no metrics crates yet)
- [Source: _bmad-output/implementation-artifacts/6-1-registry-mode-control.md] — Previous story patterns

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6 (1M context)

### Debug Log References

### Completion Notes List

- **Task 1**: Added `metrics = "0.24"` and `metrics-exporter-prometheus = "0.16"` to Cargo.toml. Added `macros` feature to axum for `FromRef` derive.
- **Task 2**: Installed Prometheus recorder in `main.rs` after tracing init. Added `describe_*!` macros for all 5 metrics (HELP/TYPE metadata). Pass handle to `api::router()`.
- **Task 3**: Created `src/api/metrics.rs` with `get_metrics` handler. Snapshots `kora_schema_count` (DB count with 2s timeout, preserves previous on failure), `kora_db_connections_in_use` (clamped to 0.0 minimum), `kora_db_connections_idle`. Returns `text/plain; version=0.0.4; charset=utf-8`.
- **Task 4**: Split router: `api` sub-router gets Confluent content-type layer, `ops` sub-router (/metrics) does not. Private `AppState` with `#[derive(FromRef)]` — existing `State<PgPool>` handler signatures unchanged (zero churn). `router()` accepts `(pool, handle, max_body_size)` — no implementation details leak to callers.
- **Task 5**: Hand-rolled `track_metrics` async middleware in `middleware.rs`. Records `http_requests_total` (counter with method/path/status labels) and `http_request_duration_seconds` (histogram with method/path). Uses `MatchedPath` for low-cardinality path labels. Skips `/metrics` to avoid self-referential feedback loop.
- **Task 6-7**: Business and pool metrics implemented as scrape-time snapshots (Option A). DB query has 2s timeout — on failure or slow query, gauge keeps previous value rather than masking an outage. Pool gauge uses `.max(0.0)` clamp against non-atomic TOCTOU race.
- **Task 8**: 8 integration tests in `tests/api_metrics.rs` covering: HTTP 200, text/plain content-type, counter presence, histogram presence, schema gauge, pool gauges, counter increment, schema count increase after registration, /metrics self-exclusion from counters. Test helper uses `OnceLock<PrometheusHandle>` with mirrored `describe_*!` calls.
- **Bonus**: Fixed pre-existing flaky test `register_level_change_to_none_allows_incompatible` — race condition caused by mutating global compat config in parallel tests. Changed to per-subject config.

### Review Record

- 4 rounds of parallel review (code review + adversarial + edge case hunter = 12 agent reviews)
- Round 1 fixes: `unwrap_or(0)` → `if let Ok` (preserve previous on DB failure), negative gauge → `.max(0.0)` clamp, misleading doc comment corrected
- Round 2 fixes: `/metrics` self-instrumentation excluded, `_total` suffix on gauge → `kora_schema_count`, `describe_*!` HELP/TYPE macros added, `describe_*!` mirrored in test helper
- Round 3 fixes: 2s timeout on DB query, Prometheus version `0.04` → `0.0.4`, self-exclusion test added, story spec synced
- Round 4: all 3 agents report no new findings — ship-ready

### Change Log

- 2026-04-15: Implemented Prometheus metrics endpoint with request instrumentation, business metrics, and pool metrics. 8 integration tests. Router split for content-type isolation. Private AppState with FromRef. 4 rounds of review (12 agent reviews), all findings resolved.

### File List

- Cargo.toml (modified) — added `metrics`, `metrics-exporter-prometheus`, `macros` feature on axum
- src/main.rs (modified) — install Prometheus recorder, `describe_*!` for all 5 metrics, pass handle to router
- src/api/mod.rs (modified) — private `AppState` with `FromRef`, split router for content-type isolation, `track_metrics` middleware layer
- src/api/metrics.rs (new) — `GET /metrics` handler with 2s-timeout DB query, `.max(0.0)` pool gauge clamp, `if let Ok` fallback
- src/api/middleware.rs (modified) — `track_metrics` request instrumentation middleware, `/metrics` self-exclusion
- tests/common/mod.rs (modified) — `prometheus_handle()` with `OnceLock`, mirrored `describe_*!`, updated `spawn_server`
- tests/api_metrics.rs (new) — 8 integration tests (endpoint basics, HTTP counters, histograms, business gauges, self-exclusion)
- tests/api_health.rs (modified) — restored to original (metrics tests moved to `api_metrics.rs`)
- tests/api_register_schema.rs (modified) — fixed flaky test: per-subject config instead of global
