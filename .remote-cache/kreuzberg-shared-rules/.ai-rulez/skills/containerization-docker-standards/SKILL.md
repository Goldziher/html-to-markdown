______________________________________________________________________

## priority: medium

# Containerization & Docker Standards

## Multi-Stage Docker Builds for Rust

- **Builder stage**: Compile with full toolchain + dependencies
- **Runtime stage**: Minimal base with only compiled binary
- **Cache optimization**: Leverage layer caching by copying dependencies before source code
- **Size reduction**: 1GB+ builder → 50-100MB runtime image

Example Dockerfile for Rust projects:

```dockerfile
# ============================================================================
# Stage 1: Builder
# ============================================================================
FROM rust:1.75-alpine AS builder

WORKDIR /build

# Install build dependencies
RUN apk add --no-cache \
    musl-dev \
    pkgconfig \
    openssl-dev \
    libssl3

# Copy manifests and lock files
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/

# Build with optimizations (single monolithic binary)
RUN RUSTFLAGS="-C target-cpu=native -C lto=fat -C codegen-units=1" \
    cargo build --release --target x86_64-unknown-linux-musl

# ============================================================================
# Stage 2: Runtime
# ============================================================================
FROM alpine:3.18

# Install minimal runtime dependencies (no build tools)
RUN apk add --no-cache \
    libssl3 \
    ca-certificates \
    tini

# Create non-root user
RUN addgroup -g 1000 appgroup && \
    adduser -D -u 1000 -G appgroup appuser

WORKDIR /app

# Copy binary from builder
COPY --from=builder --chown=appuser:appgroup \
    /build/target/x86_64-unknown-linux-musl/release/html-to-markdown \
    /app/html-to-markdown

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD /app/html-to-markdown --version || exit 1

USER appuser

ENTRYPOINT ["/sbin/tini", "--"]
CMD ["/app/html-to-markdown"]
```

## Security Standards

### Scanning & Vulnerability Detection

- **Trivy**: Fast vulnerability scanner for OS/application packages
- **Grype**: Comprehensive CVE detection
- **Snyk**: Developer-first SCA with fix suggestions
- **Scan early**: Scan base images before building

Integration in CI:

```yaml
name: Container Security

on: [push, pull_request]

jobs:
  scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Build image
        run: docker build -t myapp:latest .

      - name: Scan with Trivy
        uses: aquasecurity/trivy-action@master
        with:
          image-ref: 'myapp:latest'
          format: 'sarif'
          output: 'trivy-results.sarif'

      - name: Upload SARIF
        uses: github/codeql-action/upload-sarif@v2
        with:
          sarif_file: 'trivy-results.sarif'

      - name: Fail on high severity
        run: |
          docker run --rm -v /var/run/docker.sock:/var/run/docker.sock \
            aquasec/trivy image --severity HIGH,CRITICAL myapp:latest
```

### Minimal Base Images

**Alpine Linux** (5MB base):

- Smallest available
- musl libc instead of glibc (affects some libraries)
- Use `alpine:3.18` with clear version pin
- Lightweight package manager (apk)

**Distroless** (Google; ~20MB):

- Only app + runtime, NO shell or package manager
- Most secure; prevents interactive container access
- Available for Java, Python, Node, Go, C++, CC, base
- Cannot install additional tools (by design)

**Scratch** (0 bytes):

- Empty filesystem
- Only works for fully static binaries
- Perfect for single Go/Rust binaries compiled with `--target x86_64-unknown-linux-musl`

Example using distroless:

```dockerfile
FROM rust:1.75 AS builder
WORKDIR /build
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM gcr.io/distroless/base-debian12
COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/myapp /app
ENTRYPOINT ["/app"]
```

Example using scratch:

```dockerfile
FROM rust:1.75 AS builder
WORKDIR /build
COPY . .
RUN RUSTFLAGS="-C target-feature=+crt-static" \
    cargo build --release --target x86_64-unknown-linux-musl

FROM scratch
COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/myapp /
ENTRYPOINT ["/myapp"]
```

## Build Caching Strategies

### Docker Layer Caching

1. **Order matters**: Put stable layers first

   ```dockerfile
   # GOOD: Dependency layer cached until Cargo.lock changes
   FROM rust:1.75 AS builder
   WORKDIR /build
   COPY Cargo.toml Cargo.lock ./
   RUN cargo fetch  # Downloads all dependencies
   COPY src/ ./src/
   RUN cargo build --release

   # BAD: Rebuilds dependencies on every source change
   COPY . .
   RUN cargo build --release
   ```

1. **BuildKit inline cache**: More efficient than standard Docker

   ```bash
   docker buildx build --push \
     --cache-from=type=registry,ref=myregistry/myapp:cache \
     --cache-to=type=registry,ref=myregistry/myapp:cache,mode=max \
     -t myregistry/myapp:latest .
   ```

1. **Mount cache volumes** (BuildKit):

   ```dockerfile
   # Use cached cargo registry across builds
   RUN --mount=type=cache,target=/usr/local/cargo/registry \
       --mount=type=cache,target=/build/target \
       cargo build --release
   ```

### GitHub Actions BuildKit Cache

```yaml
- name: Build and push
  uses: docker/build-push-action@v5
  with:
    push: ${{ github.event_name == 'push' }}
    tags: myregistry/myapp:latest
    cache-from: type=gha
    cache-to: type=gha,mode=max
```

## Image Tagging Strategy

- **Semantic versioning**: `myregistry/myapp:1.2.3`
- **Latest**: `myregistry/myapp:latest` (use sparingly, tag after release)
- **Branch**: `myregistry/myapp:main`, `myregistry/myapp:develop`
- **Commit SHA**: `myregistry/myapp:abc1234f` (for traceability)
- **Combination**: Tag single build with multiple tags

```bash
# Tag one build with multiple tags
docker build -t myapp:1.2.3 -t myapp:latest -t myapp:main .
docker push myapp:1.2.3 && docker push myapp:latest && docker push myapp:main
```

## Docker Compose for Development

Example docker-compose.yml:

```yaml
version: '3.9'

services:
  app:
    build:
      context: .
      dockerfile: Dockerfile.dev
    environment:
      RUST_LOG: debug
      DATABASE_URL: postgresql://user:pass@postgres:5432/db
    ports:
      - "3000:3000"
    depends_on:
      postgres:
        condition: service_healthy
    volumes:
      - .:/app
    command: cargo run

  postgres:
    image: postgres:15-alpine
    environment:
      POSTGRES_PASSWORD: password
      POSTGRES_DB: db
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U user"]
      interval: 10s
      timeout: 5s
      retries: 5
    volumes:
      - postgres_data:/var/lib/postgresql/data

volumes:
  postgres_data:
```

## Best Practices

1. **Non-root user**: Always run container as non-root
1. **Health checks**: Include HEALTHCHECK for orchestrators
1. **Signal handling**: Use `tini` or `dumb-init` as PID 1
1. **Read-only filesystem**: Run with `--read-only` when possible
1. **Resource limits**: Set CPU/memory requests and limits
1. **No secrets in image**: Use environment variables or secret mounts
1. **Immutable tags**: Never reuse tags after push; use commit SHAs

## Anti-Patterns

- **Running as root**: Creates security risk
- **Ignore .dockerignore**: Bloats build context
- **Installing unnecessary packages**: Alpine bloat vs. distroless speed
- **Hardcoding secrets**: Use BuildKit secrets: `docker build --secret mytoken`
- **No HEALTHCHECK**: Prevents Kubernetes/Swarm from detecting failures
- **Large base images**: Using ubuntu:22.04 (77MB) instead of alpine (5MB)
- **Single stage builds**: Defeats layer caching and size optimization
- **Not using BuildKit**: Miss out on parallelization and inline caches
- **Logging to stdout**: Applications must log to stdout/stderr for Docker capture
