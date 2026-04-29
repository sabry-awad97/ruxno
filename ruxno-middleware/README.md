# Ruxno Middleware

Optional middleware collection for the [Ruxno](../ruxno) web framework.

## Features

This crate provides production-ready middleware that can be easily integrated into Ruxno applications. Each middleware is feature-gated, so you only pay for what you use.

### Available Middleware

- **rate-limit**: In-memory rate limiting using `governor`
- **rate-limit-redis**: Distributed rate limiting with Redis
- **cors**: Cross-Origin Resource Sharing (CORS) support
- **compression**: Response compression (gzip, brotli)
- **auth**: JWT authentication helpers
- **logger**: Request/response logging
- **security-headers**: Security headers (HSTS, CSP, etc.)

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ruxno = "0.1"
ruxno-middleware = { version = "0.1", features = ["rate-limit", "cors"] }
```

### Feature Flags

- `default`: No middleware enabled by default
- `rate-limit`: In-memory rate limiting
- `rate-limit-redis`: Distributed rate limiting with Redis
- `cors`: CORS support
- `compression`: Response compression
- `auth`: JWT authentication
- `logger`: Request/response logging
- `security-headers`: Security headers
- `full`: Enable all middleware
- `production`: Enable production-ready middleware (rate-limit, cors, security-headers)

## Usage

### Rate Limiting

```rust
use ruxno::App;
use ruxno_middleware::RateLimitMiddleware;

let mut app = App::new();

// 100 requests per second per IP
let rate_limiter = RateLimitMiddleware::per_second(100);
app.use_middleware("*", rate_limiter.middleware());
```

### CORS (Coming Soon)

```rust
use ruxno::App;
use ruxno_middleware::CorsMiddleware;

let mut app = App::new();

let cors = CorsMiddleware::new()
    .allow_origin("https://example.com")
    .allow_methods(vec!["GET", "POST"])
    .allow_credentials(true);

app.use_middleware("*", cors.middleware());
```

## Development Status

- ✅ **rate-limit**: Basic implementation complete
- 🚧 **cors**: Placeholder (coming soon)
- 🚧 **compression**: Placeholder (coming soon)
- 🚧 **auth**: Placeholder (coming soon)
- 🚧 **logger**: Placeholder (coming soon)
- 🚧 **security-headers**: Placeholder (coming soon)

## Contributing

Contributions are welcome! Please see the main [Ruxno repository](https://github.com/yourusername/ruxno) for contribution guidelines.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](../LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
