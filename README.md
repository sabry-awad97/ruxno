# Ruxno

A fast, ergonomic Rust web framework with clean architecture.

## Workspace Structure

This repository is organized as a Cargo workspace:

- **[ruxno](./ruxno)**: Core framework (lean, minimal dependencies)
- **[ruxno-middleware](./ruxno-middleware)**: Optional middleware collection
- **[examples](./examples)**: Example applications demonstrating framework features

## Quick Start

### Core Framework Only

```toml
[dependencies]
ruxno = "0.1"
tokio = { version = "1", features = ["full"] }
```

```rust
use ruxno::prelude::*;

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.get("/", async |ctx: Context| {
        Ok(ctx.text("Hello, World!"))
    });

    app.listen("127.0.0.1:3000").await.unwrap();
}
```

### With Middleware

```toml
[dependencies]
ruxno = "0.1"
ruxno-middleware = { version = "0.1", features = ["rate-limit"] }
tokio = { version = "1", features = ["full"] }
```

```rust
use ruxno::prelude::*;
use ruxno_middleware::RateLimitMiddleware;

#[tokio::main]
async fn main() {
    let mut app = App::new();

    // Add rate limiting (100 requests per second per IP)
    let rate_limiter = RateLimitMiddleware::per_second(100);
    app.r#use(rate_limiter);

    app.get("/", async |ctx: Context| {
        Ok(ctx.text("Hello, World!"))
    });

    app.listen("127.0.0.1:3000").await.unwrap();
}
```

## Features

### Core Framework (ruxno)

- ✅ Clean architecture (zero dependencies in core layer)
- ✅ Fast routing with radix tree
- ✅ Middleware system
- ✅ WebSocket support
- ✅ Server-Sent Events (SSE)
- ✅ Body parsing (JSON, form, multipart)
- ✅ Production-ready error handling
- ✅ Graceful shutdown
- ✅ Request timeouts
- ✅ Connection limits

### Middleware Collection (ruxno-middleware)

All middleware modules are implemented and available via feature flags:

- ✅ **rate-limit**: In-memory rate limiting using `governor`
- ✅ **cors**: Cross-Origin Resource Sharing (CORS) support
- ✅ **compression**: Response compression (gzip, brotli)
- ✅ **auth**: JWT authentication helpers
- ✅ **logger**: Request/response logging with `tracing`
- ✅ **security-headers**: Security headers (HSTS, CSP, etc.)

Enable all middleware with the `full` feature:

```toml
ruxno-middleware = { version = "0.1", features = ["full"] }
```

Or enable specific middleware:

```toml
ruxno-middleware = { version = "0.1", features = ["rate-limit", "cors", "logger"] }
```

## Documentation

- [Core Framework Documentation](./ruxno/README.md)
- [Middleware Documentation](./ruxno-middleware/README.md)
- [API Documentation](https://docs.rs/ruxno)

## Development

### Building the Workspace

```bash
# Build all crates
cargo build

# Build specific crate
cargo build -p ruxno
cargo build -p ruxno-middleware

# Run tests
cargo test

# Run benchmarks
cargo bench
```

### Project Structure

```
ruxno-clean/
├── Cargo.toml                      # Workspace configuration
├── README.md                       # This file
├── CHECKLIST.md                    # Development checklist
├── examples/                       # Example applications
│   ├── 01_hello_world/            # Basic hello world example
│   │   ├── src/main.rs
│   │   ├── request.http           # REST client tests
│   │   └── Cargo.toml
│   └── 02_middleware_patterns/    # Middleware patterns demo
│       ├── src/main.rs
│       ├── request.http           # REST client tests
│       └── Cargo.toml
├── ruxno/                          # Core framework
│   ├── Cargo.toml
│   ├── README.md
│   ├── benches/                    # Performance benchmarks
│   ├── examples/                   # Inline examples
│   ├── tests/                      # Integration tests
│   └── src/
│       ├── core/                   # Zero dependencies
│       ├── domain/                 # Business models
│       ├── routing/                # Pure routing logic
│       ├── pipeline/               # Middleware orchestration
│       ├── http/                   # HTTP protocol layer
│       ├── body/                   # Body parsing
│       ├── upgrade/                # WebSocket/SSE
│       ├── server/                 # HTTP server
│       ├── app/                    # Application builder
│       ├── prelude.rs              # Common imports
│       └── lib.rs
└── ruxno-middleware/               # Optional middleware
    ├── Cargo.toml
    ├── README.md
    └── src/
        ├── rate_limit.rs           # Rate limiting
        ├── cors.rs                 # CORS support
        ├── compression.rs          # Response compression
        ├── auth.rs                 # JWT authentication
        ├── logger.rs               # Request logging
        ├── security_headers.rs     # Security headers
        └── lib.rs
```

## Examples

The workspace includes example applications demonstrating framework features:

### 01_hello_world

Basic example showing:

- Route registration with multiple HTTP methods
- Global and path-specific middleware
- Route builder pattern
- Path parameters
- REST client test file (`request.http`)

Run with:

```bash
cd examples/01_hello_world
cargo run
```

### 02_middleware_patterns

Demonstrates middleware patterns:

- Global middleware (applies to all routes)
- Method-specific middleware (e.g., POST only)
- Path-specific middleware (e.g., `/api/*`)
- Middleware execution order
- REST client test file (`request.http`)

Run with:

```bash
cd examples/02_middleware_patterns
cargo run
```

## Testing Examples

Each example includes a `request.http` file for testing with REST Client (VS Code extension) or similar tools. These files contain pre-configured HTTP requests to test all endpoints.

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](./CONTRIBUTING.md) for guidelines.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](./LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](./LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
