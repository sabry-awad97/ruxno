# Contributing to Ruxno

Thank you for your interest in contributing to Ruxno! This document provides guidelines and instructions for contributing to the project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Project Structure](#project-structure)
- [Coding Guidelines](#coding-guidelines)
- [Testing](#testing)
- [Submitting Changes](#submitting-changes)
- [Documentation](#documentation)

## Code of Conduct

We are committed to providing a welcoming and inclusive environment. Please be respectful and constructive in all interactions.

## Getting Started

### Prerequisites

- **Rust 1.85+** (required for async closure support)
- **Cargo** (comes with Rust)
- **Git**

### Fork and Clone

1. Fork the repository on GitHub
2. Clone your fork:
   ```bash
   git clone https://github.com/YOUR_USERNAME/ruxno.git
   cd ruxno/ruxno-clean
   ```
3. Add upstream remote:
   ```bash
   git remote add upstream https://github.com/sabry-awad97/ruxno.git
   ```

## Development Setup

### Building the Project

```bash
# Build all workspace members
cargo build

# Build specific crate
cargo build -p ruxno
cargo build -p ruxno-middleware

# Build with all features
cargo build --all-features

# Build in release mode
cargo build --release
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests for specific crate
cargo test -p ruxno
cargo test -p ruxno-middleware

# Run tests with all features
cargo test --all-features

# Run specific test
cargo test test_name
```

### Running Examples

```bash
# Run hello world example
cd examples/01_hello_world
cargo run

# Run middleware patterns example
cd examples/02_middleware_patterns
cargo run
```

### Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench --bench routing
```

## Project Structure

```
ruxno-clean/
├── ruxno/                  # Core framework
│   ├── src/
│   │   ├── core/          # Zero-dependency abstractions
│   │   ├── domain/        # Business models (Request, Response, Context)
│   │   ├── routing/       # Pure routing logic
│   │   ├── pipeline/      # Middleware orchestration
│   │   ├── http/          # HTTP protocol layer
│   │   ├── body/          # Body parsing
│   │   ├── upgrade/       # WebSocket/SSE support
│   │   ├── server/        # HTTP server implementation
│   │   ├── app/           # Application builder (public API)
│   │   └── prelude.rs     # Common imports
│   ├── benches/           # Performance benchmarks
│   ├── examples/          # Inline examples
│   └── tests/             # Integration tests
├── ruxno-middleware/       # Optional middleware collection
│   └── src/
│       ├── rate_limit.rs
│       ├── cors.rs
│       ├── compression.rs
│       ├── auth.rs
│       ├── logger.rs
│       └── security_headers.rs
└── examples/               # Example applications
    ├── 01_hello_world/
    └── 02_middleware_patterns/
```

### Architecture Layers

Ruxno follows a clean architecture with clear layer boundaries:

1. **Core Layer** (`src/core/`) - Zero dependencies, pure abstractions
2. **Domain Layer** (`src/domain/`) - Business models and logic
3. **Routing Layer** (`src/routing/`) - Pure routing logic
4. **Pipeline Layer** (`src/pipeline/`) - Middleware orchestration
5. **Infrastructure Layer** (`src/http/`, `src/server/`) - External dependencies
6. **Application Layer** (`src/app/`) - Public API facade

**Rule**: Inner layers must not depend on outer layers.

## Coding Guidelines

### Rust Version and Features

- **Minimum Rust version**: 1.85 (for async closure support)
- Use `async |ctx: Context| { }` syntax for handlers
- Use `#[async_trait]` for trait objects requiring async methods
- See [AGENTS.md](./AGENTS.md) for detailed async patterns

### Code Style

We follow standard Rust conventions:

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt -- --check

# Run clippy
cargo clippy --all-targets --all-features

# Fix clippy warnings
cargo clippy --fix
```

### Naming Conventions

Based on actual codebase patterns:

- **Types**: `PascalCase` (e.g., `Context`, `Response`, `RateLimitMiddleware`)
- **Functions/methods**: `snake_case` (e.g., `listen`, `with_header`, `per_second`)
- **Constants**: `SCREAMING_SNAKE_CASE` (e.g., `VERSION`)
- **Modules**: `snake_case` (e.g., `rate_limit`, `security_headers`)

### Type Conversions

**Always use built-in conversion traits:**

```rust
// ✅ CORRECT: Use From/Into
impl From<HyperRequest> for Request {
    fn from(req: HyperRequest) -> Self {
        Request::new(req)
    }
}

// ✅ CORRECT: Use TryFrom for fallible conversions
impl TryFrom<String> for Email {
    type Error = ValidationError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        // validation logic
    }
}

// ❌ WRONG: Custom conversion methods
impl Request {
    pub fn from_hyper(req: HyperRequest) -> Self { }
}
```

### Error Handling

- Use `Result<T, CoreError>` for fallible operations
- Use `thiserror` for custom error types
- Provide helpful error messages
- Don't panic in library code (use `Result` instead)

### Middleware Implementation

Middleware should implement the `Middleware` trait:

```rust
use async_trait::async_trait;
use ruxno::core::{CoreError, Middleware, Next};
use ruxno::{Context, Response};

pub struct MyMiddleware {
    // fields
}

#[async_trait]
impl<E> Middleware<E> for MyMiddleware
where
    E: Send + Sync + 'static,
{
    async fn process(&self, ctx: Context<E>, next: Next<E>) -> Result<Response, CoreError> {
        // Before handler
        println!("Before");

        // Call next handler
        let response = next.run(ctx).await?;

        // After handler
        println!("After");
        Ok(response)
    }
}
```

### Documentation

- Add doc comments to all public items
- Use `///` for item documentation
- Use `//!` for module documentation
- Include examples in doc comments when helpful
- Run `cargo doc --open` to preview documentation

Example:

````rust
/// Rate limiting middleware using the token bucket algorithm.
///
/// # Examples
///
/// ```rust,ignore
/// use ruxno_middleware::RateLimitMiddleware;
///
/// let limiter = RateLimitMiddleware::per_second(100);
/// app.r#use(limiter);
/// ```
pub struct RateLimitMiddleware {
    // ...
}
````

## Testing

### Writing Tests

- Place unit tests in the same file as the code being tested
- Place integration tests in the `tests/` directory
- Use descriptive test names: `test_<what>_<condition>_<expected>`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_creation() {
        let limiter = RateLimitMiddleware::per_second(100);
        // assertions
    }

    #[tokio::test]
    async fn test_middleware_process() {
        // async test
    }
}
```

### Test Coverage

- Aim for high test coverage on core functionality
- Test edge cases and error conditions
- Test public API thoroughly

## Submitting Changes

### Branch Naming

- `feature/description` - New features
- `fix/description` - Bug fixes
- `docs/description` - Documentation changes
- `refactor/description` - Code refactoring
- `test/description` - Test additions/changes

### Commit Messages

Follow conventional commits format:

```
type(scope): brief description

Longer description if needed.

Fixes #123
```

Types:

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Test additions/changes
- `chore`: Build process or auxiliary tool changes

Examples:

```
feat(middleware): add compression middleware

Implements gzip and brotli compression for responses.

Fixes #42
```

```
fix(routing): handle trailing slashes correctly

Routes with and without trailing slashes now match correctly.

Fixes #56
```

### Pull Request Process

1. **Update your fork**:

   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

2. **Create a feature branch**:

   ```bash
   git checkout -b feature/my-feature
   ```

3. **Make your changes**:
   - Follow coding guidelines
   - Add tests
   - Update documentation
   - Run `cargo fmt` and `cargo clippy`

4. **Commit your changes**:

   ```bash
   git add .
   git commit -m "feat(scope): description"
   ```

5. **Push to your fork**:

   ```bash
   git push origin feature/my-feature
   ```

6. **Create a Pull Request**:
   - Go to GitHub and create a PR from your fork
   - Fill out the PR template
   - Link related issues
   - Wait for review

### PR Checklist

Before submitting a PR, ensure:

- [ ] Code follows project style guidelines
- [ ] All tests pass (`cargo test`)
- [ ] Code is formatted (`cargo fmt`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Documentation is updated
- [ ] Examples are updated if API changed
- [ ] Commit messages follow conventions
- [ ] PR description explains the changes

## Documentation

### Inline Documentation

- Document all public APIs
- Include examples in doc comments
- Explain complex algorithms or patterns
- Document safety requirements for unsafe code

### README Updates

Update relevant README files when:

- Adding new features
- Changing public APIs
- Adding new examples
- Updating dependencies

### Examples

When adding features:

- Add or update examples in `examples/`
- Include `request.http` files for REST testing
- Document usage in example comments

## Feature Flags

When adding middleware to `ruxno-middleware`:

1. Add feature flag in `Cargo.toml`:

   ```toml
   [features]
   my-middleware = ["dependency"]
   ```

2. Gate the module:

   ```rust
   #[cfg(feature = "my-middleware")]
   pub mod my_middleware;
   ```

3. Add to `full` feature bundle:

   ```toml
   full = ["rate-limit", "cors", "my-middleware", ...]
   ```

4. Update documentation in `lib.rs`

## Getting Help

- Open an issue for bugs or feature requests
- Start a discussion for questions or ideas
- Check existing issues and PRs before creating new ones

## License

By contributing to Ruxno, you agree that your contributions will be licensed under both:

- MIT License
- Apache License 2.0

Thank you for contributing to Ruxno! 🚀
