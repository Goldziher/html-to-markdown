______________________________________________________________________

## priority: medium

# Monitoring & Observability Standards

## Structured Logging with Tracing Crate

- **Use `tracing` crate**: Unified logging, tracing, and metrics in Rust
- **Structured events**: key=value pairs instead of f-string format
- **Span context**: Automatic propagation of request IDs, user info, etc.
- **Multiple subscribers**: Layer logs, metrics, and distributed traces together

Basic setup in Rust:

```rust
use tracing::{debug, info, warn, error, instrument, span, Level};
use tracing_subscriber::{fmt, prelude::*};

#[tokio::main]
async fn main() {
    // Initialize tracing with multiple layers
    tracing_subscriber::registry()
        .with(
            fmt::layer()
                .with_writer(std::io::stdout)
                .with_target(true)
                .with_thread_ids(true)
                .with_level(true)
                .json()  // Structured JSON output for log aggregation
        )
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".parse().unwrap())
        )
        .init();

    info!("Application starting", version = env!("CARGO_PKG_VERSION"));

    // Run application
    run().await;
}

#[instrument(skip(client), fields(user_id = %user_id))]
async fn process_request(user_id: i64, client: &HttpClient) -> Result<String> {
    debug!("Processing user request");

    let response = client.get(&format!("/users/{}", user_id))
        .await
        .map_err(|e| {
            error!(error = ?e, "Failed to fetch user");
            e
        })?;

    info!(status = response.status().as_u16(), "Request successful");
    Ok(response.text().await?)
}
```

## Span Instrumentation

- **Automatic span creation**: Use `#[instrument]` macro on functions
- **Manual spans**: Create spans for logical operations
- **Nested spans**: Parent-child relationship tracked automatically
- **Span fields**: Add context about operation (user_id, request_id, etc.)

```rust
use tracing::{Span, span, Level};

#[instrument(skip(db, cache))]
async fn fetch_user_data(
    user_id: i64,
    db: &Database,
    cache: &Cache,
) -> Result<User> {
    // Span created automatically from function name
    // Fields: user_id added to span context

    let cache_span = span!(Level::DEBUG, "cache_lookup", user_id = %user_id);
    let _enter = cache_span.enter();

    if let Some(user) = cache.get(user_id).await {
        debug!("Cache hit");
        return Ok(user);
    }

    drop(_enter);  // Exit cache span

    // Switch to database lookup
    let db_span = span!(Level::DEBUG, "db_lookup", user_id = %user_id);
    let _enter = db_span.enter();

    let user = db.query_user(user_id).await?;
    cache.set(user_id, user.clone()).await;

    info!("User loaded from database");
    Ok(user)
}
```

## Log Levels

- **ERROR**: Application error, unrecoverable (invalid request, DB connection failed)
- **WARN**: Unexpected condition, but application continues (deprecated API usage, missing optional config)
- **INFO**: Important state changes (startup complete, request processed, migration started)
- **DEBUG**: Detailed execution flow for debugging (entering function, variable values)
- **TRACE**: Very detailed, typically off in production (field assignments, loop iterations)

Best practices:

```rust
// ERROR: User made invalid request or system failure
error!(error = ?e, "Failed to process payment");

// WARN: Degraded service or unusual state
warn!(deprecated_field = true, "Using deprecated API");

// INFO: Progress milestones
info!(user_count = count, "Database migration completed");

// DEBUG: Entry/exit and important values
debug!("Entering user validation");
debug!(validation_result = ?result, "Validation complete");

// TRACE: Detailed loops and assignments (usually disabled)
trace!(index = i, value = ?item, "Processing item");
```

## Python Structured Logging

Use `structlog` for structured logging in Python:

```python
import structlog
from typing import Any

# Configure structlog
structlog.configure(
    processors=[
        structlog.processors.TimeStamper(fmt="iso"),
        structlog.processors.JSONRenderer()
    ],
    context_class=dict,
    logger_factory=structlog.PrintLoggerFactory(),
    cache_logger_on_first_use=True,
)

logger = structlog.get_logger()

def process_request(user_id: int) -> dict[str, Any]:
    logger.msg("Processing user request", user_id=user_id)

    try:
        result = fetch_user_data(user_id)
        logger.msg("Request successful", user_id=user_id, status=200)
        return result
    except Exception as e:
        logger.exception("Request failed", user_id=user_id, error=str(e))
        raise
```

## Metrics Collection with Prometheus

- **Counter**: Monotonically increasing value (requests, errors, bytes sent)
- **Gauge**: Current value that can go up/down (CPU usage, queue size, connections)
- **Histogram**: Distribution of values (request latency, payload size)
- **Summary**: Percentiles of values (same as histogram, different output format)

Rust with `prometheus` crate:

```rust
use prometheus::{
    Counter, CounterVec, Gauge, Histogram, HistogramVec, Registry,
};
use once_cell::sync::Lazy;

// Define metrics
pub static REQUEST_COUNTER: Lazy<CounterVec> = Lazy::new(|| {
    CounterVec::new(
        prometheus::Opts::new("http_requests_total", "Total HTTP requests"),
        &["method", "status"],
    ).unwrap()
});

pub static REQUEST_DURATION: Lazy<HistogramVec> = Lazy::new(|| {
    HistogramVec::new(
        prometheus::HistogramOpts::new(
            "http_request_duration_seconds",
            "HTTP request duration in seconds"
        ),
        &["method", "path"],
    ).unwrap()
});

pub static ACTIVE_CONNECTIONS: Lazy<Gauge> = Lazy::new(|| {
    Gauge::new("connections_active", "Active connections").unwrap()
});

// Use metrics in handler
#[instrument(skip(db))]
async fn handle_request(method: &str, path: &str, db: &Database) -> Result<Response> {
    let timer = REQUEST_DURATION.with_label_values(&[method, path]).start_timer();

    ACTIVE_CONNECTIONS.inc();

    let result = process_request(db).await;

    ACTIVE_CONNECTIONS.dec();

    let status = if result.is_ok() { "200" } else { "500" };
    REQUEST_COUNTER.with_label_values(&[method, status]).inc();

    timer.observe_duration();
    result
}
```

Prometheus scrape config (prometheus.yml):

```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'myapp'
    static_configs:
      - targets: ['localhost:9090']
    relabel_configs:
      - source_labels: [__address__]
        target_label: instance
```

## Integration with Cloud Observability

### Datadog

```rust
use ddtrace::Tracer;

let tracer = Tracer::new();

#[instrument(skip(db))]
async fn fetch_data(db: &Database) {
    // Automatically traced and sent to Datadog
}
```

### New Relic

```rust
use newrelic::{Config, App};

let config = Config::new("My App", "license-key");
let app = App::new(config)?;

let txn = app.start_transaction("fetch_user");
// Work happens here
txn.notice_error(error);
```

### Google Cloud Trace

```rust
use opentelemetry_gcp::trace::CloudTraceExporter;
use opentelemetry_sdk::trace::TracerProvider;

let exporter = CloudTraceExporter::new();
let provider = TracerProvider::builder()
    .with_batch_exporter(exporter)
    .build();

let tracer = provider.tracer("myapp");
```

## Health Check Endpoints

Always expose a `/health` endpoint for orchestrators:

```rust
#[get("/health")]
async fn health(db: &State<Database>) -> Json<HealthStatus> {
    let db_healthy = db.ping().await.is_ok();
    let status = if db_healthy { "healthy" } else { "unhealthy" };

    Json(HealthStatus {
        status,
        checks: json!({
            "database": db_healthy,
            "uptime_seconds": uptime(),
        }),
    })
}

// Kubernetes liveness probe
# pod.yaml
livenessProbe:
  httpGet:
    path: /health
    port: 3000
  initialDelaySeconds: 10
  periodSeconds: 10
```

## Distributed Tracing

Propagate trace IDs across services using `W3C Trace Context`:

```rust
use http::HeaderMap;
use opentelemetry::api::TraceContextPropagator;

async fn call_downstream_service(
    headers: &HeaderMap,
    client: &HttpClient,
) -> Result<Response> {
    // Extract parent trace context from incoming request
    let propagator = TraceContextPropagator::new();
    let parent_context = propagator.extract(headers);

    let span = span!(
        parent: &parent_context,
        tracing::Level::DEBUG,
        "downstream_call",
        service = "user-service"
    );

    let _enter = span.enter();

    // Inject trace context into outgoing request
    let mut outgoing_headers = HeaderMap::new();
    propagator.inject_context(&parent_context, &mut outgoing_headers);

    client.get("/api/users")
        .headers(outgoing_headers)
        .send()
        .await
}
```

## Anti-Patterns

- **f-string logging**: Use structured key=value instead

  ```rust
  // BAD
  info!("User {} processed with status {}", user_id, status);

  // GOOD
  info!("User processed", user_id = user_id, status = status);
  ```

- **No span context**: Always wrap request handling in spans with request ID

- **Synchronous logging in hot path**: Use async subscribers in high-throughput services

- **Hardcoded log levels**: Respect environment variable configuration

- **Logging sensitive data**: Never log passwords, tokens, PII without redaction

- **No metrics**: Always instrument critical paths (requests, errors, latency)

- **High cardinality labels**: Avoid unbounded label values (user_id in label)

  ```rust
  // BAD: Unbounded cardinality
  REQUEST_COUNTER.with_label_values(&[method, &user_id.to_string()]).inc();

  // GOOD: Fixed cardinality
  REQUEST_COUNTER.with_label_values(&[method, "success"]).inc();
  ```

- **No health checks**: Orchestrators can't detect unhealthy instances

- **Sampling off**: Use tail-based sampling in production for cost
