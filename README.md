# Ruxno

A fast, ergonomic Rust web framework with clean architecture.

## Workspace Structure

This repository is organized as a Cargo workspace with two crates:

- **[ruxno](./ruxno)**: Core framework (lean, minimal dependencies)
- **[ruxno-middleware](./ruxno-middleware)**: Optional middleware collection

## Quick Start

### Core Framework Only

```toml
[dependencies]
ruxno = "0.1"
```

```rust
use ruxno::App;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new();

    app.get("/", |ctx| async move {
        Ok(ctx.text("Hello, World!"))
    });

    app.listen("127.0.0.1:3000").await?;
    Ok(())
}
```

### With Middleware

```toml
[dependencies]
ruxno = "0.1"
ruxno-middleware = { version = "0.1", features = ["rate-limit", "cors"] }
```

```rust
use ruxno::App;
use ruxno_middleware::RateLimitMiddleware;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new();

    // Add rate limiting
    let rate_limiter = RateLimitMiddleware::per_second(100);
    app.use_middleware("*", rate_limiter.middleware());

    app.get("/", |ctx| async move {
        Ok(ctx.text("Hello, World!"))
    });

    app.listen("127.0.0.1:3000").await?;
    Ok(())
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

- ✅ Rate limiting (in-memory)
- 🚧 CORS (coming soon)
- 🚧 Compression (coming soon)
- 🚧 Authentication (coming soon)
- 🚧 Logging (coming soon)
- 🚧 Security headers (coming soon)

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
├── Cargo.toml              # Workspace configuration
├── README.md               # This file
├── ruxno/                  # Core framework
│   ├── Cargo.toml
│   ├── src/
│   │   ├── core/           # Zero dependencies
│   │   ├── domain/         # Business models
│   │   ├── routing/        # Pure routing logic
│   │   ├── pipeline/       # Middleware orchestration
│   │   ├── http/           # HTTP protocol layer
│   │   ├── body/           # Body parsing
│   │   ├── upgrade/        # WebSocket/SSE
│   │   ├── server/         # HTTP server
│   │   └── app/            # Application builder
│   └── examples/
└── ruxno-middleware/       # Optional middleware
    ├── Cargo.toml
    ├── src/
    │   ├── rate_limit.rs
    │   ├── cors.rs
    │   ├── compression.rs
    │   ├── auth.rs
    │   ├── logger.rs
    │   └── security_headers.rs
    └── README.md
```

## Production Readiness

The core framework has been hardened for production use:

- ✅ All critical security vulnerabilities fixed (5/5)
- ✅ All high severity issues resolved (5/5)
- ✅ 559 tests passing
- ✅ Comprehensive error handling
- ✅ Graceful shutdown
- ✅ Resource limits and timeouts

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](./CONTRIBUTING.md) for guidelines.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](./LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](./LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
