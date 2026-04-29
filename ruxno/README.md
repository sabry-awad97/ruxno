# Ruxno

A fast, ergonomic Rust web framework with clean architecture principles.

## Architecture

Ruxno follows a layered clean architecture:

```
┌─────────────────────────────────────────┐
│         Application (App Facade)        │  ← Public API
├─────────────────────────────────────────┤
│      Pipeline (Dispatcher/Executor)     │  ← Orchestration
├─────────────────────────────────────────┤
│    Domain (Request/Response/Context)    │  ← Business Models
├─────────────────────────────────────────┤
│      Core (Handler/Middleware Traits)   │  ← Abstractions
├─────────────────────────────────────────┤
│   Infrastructure (Server/HTTP/Hyper)    │  ← External Dependencies
└─────────────────────────────────────────┘
```

## Features

- **Clean Architecture**: Layered design with clear separation of concerns
- **Performance**: Pre-computed middleware chains, zero-copy requests
- **Type Safety**: Generic environment parameters, type-safe extensions
- **Testability**: Each layer can be tested independently
- **Ergonomic API**: Hono.js-inspired API for familiarity

## Quick Start

```rust
use ruxno::prelude::*;

#[tokio::main]
async fn main() {
    let mut app = App::new();

    app.get("/", async |c: Context| {
        Ok(c.text("Hello, World!"))
    });

    app.listen("127.0.0.1:3000").await.unwrap();
}
```

## License

MIT OR Apache-2.0
