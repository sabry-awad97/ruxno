# Hello World Example

A comprehensive example demonstrating Ruxno's core features.

## Features Demonstrated

- 🔍 **HTTP Sniffer middleware** - Detailed request logging similar to Node.js HTTP sniffing utilities
- ✨ **Pretty JSON middleware** - Automatic JSON formatting with 2-space indentation
- 🔍 **Global middleware** - Request/response logging for all routes
- 🔐 **Path-specific middleware** - Middleware that only applies to specific routes
- 🛣️ **Route builder pattern** - Fluent API for defining routes
- 📝 **Path parameters** - Dynamic route segments (e.g., `/users/:id`)
- 📊 **Beautiful tables** - Startup information displayed in formatted tables

## Running the Example

### Start the Server

```bash
cargo run
```

The server will start on `http://127.0.0.1:3000` and display:

- Features table
- Routes table
- Middleware patterns table

**Note:** The HTTP sniffer middleware will log detailed information about every request, including:

- Timestamp (ISO 8601 format)
- HTTP method and version
- Request URL and parsed components
- All headers with enumerated output
- Query parameters

Watch the console output to see the detailed request logs!

### Test with the Client

In another terminal, run the test client:

```bash
cargo run --bin test_client
```

This will test all endpoints and display the responses.

### Test with REST Client

Open `request.http` in VS Code with the REST Client extension and click "Send Request" on any endpoint.

### Test with curl

```bash
# Home endpoint
curl http://127.0.0.1:3000/

# List users
curl http://127.0.0.1:3000/users

# Create user
curl -X POST http://127.0.0.1:3000/users \
  -H "Content-Type: application/json" \
  -d '{"name":"Alice","email":"alice@example.com"}'

# Get user by ID
curl http://127.0.0.1:3000/users/123

# Update user
curl -X PUT http://127.0.0.1:3000/users/123 \
  -H "Content-Type: application/json" \
  -d '{"name":"Alice Updated","email":"alice.updated@example.com"}'

# Delete user
curl -X DELETE http://127.0.0.1:3000/users/123

# Admin dashboard
curl http://127.0.0.1:3000/admin

# Admin action
curl -X POST http://127.0.0.1:3000/admin \
  -H "Content-Type: application/json" \
  -d '{"action":"update_settings"}'

# API status
curl http://127.0.0.1:3000/api/status
```

## Available Endpoints

| Method | Path          | Description                 | Middleware     |
| ------ | ------------- | --------------------------- | -------------- |
| GET    | `/`           | Home endpoint with API info | Global         |
| GET    | `/users`      | List all users              | Global         |
| POST   | `/users`      | Create a new user           | Global         |
| GET    | `/users/:id`  | Get user by ID              | Global         |
| PUT    | `/users/:id`  | Update user                 | Global         |
| DELETE | `/users/:id`  | Delete user                 | Global         |
| GET    | `/admin`      | Admin dashboard             | Global + Admin |
| POST   | `/admin`      | Admin action                | Global + Admin |
| GET    | `/api/status` | API status                  | Global + API   |

## Middleware Patterns

- **HTTP Sniffer** - Logs detailed request information for all routes
  - Captures HTTP method, version, URL, headers, and query parameters
  - Useful for debugging and monitoring

- **Global (`*`)** - Applies to all routes
  - Logs all requests and responses
  - Pretty-prints all JSON responses

- **API (`/api/*`)** - Applies only to API routes
  - Additional authentication check (placeholder)

- **Admin (`/admin`)** - Applies only to admin routes
  - Admin authentication check (placeholder)

## Project Structure

```
01_hello_world/
├── src/
│   ├── main.rs           # Main server application
│   ├── util.rs           # Utility functions (table printing)
│   └── bin/
│       └── test_client.rs # Test client using reqwest
├── request.http          # REST Client test file
├── Cargo.toml
└── README.md
```

## Code Highlights

### HTTP Sniffer Middleware

```rust
use crate::middleware::http_sniffer::HttpSnifferExt;

app.with_http_sniffer();
```

Logs detailed request information including method, version, URL, headers, and query parameters.

### Pretty JSON Middleware

```rust
use ruxno_middleware::pretty_json;

app.r#use(pretty_json());
```

All JSON responses are automatically formatted with 2-space indentation.

### Global Middleware

```rust
app.r#use(async |ctx: Context, next: Next| {
    println!("🔍 Request: {} {}", ctx.req.method().as_str(), ctx.req.path());
    let response = next.run(ctx).await?;
    println!("✅ Response: {}", response.status());
    Ok(response)
});
```

### Path-Specific Middleware

```rust
app.on(Method::GET, "/api/*", async |ctx: Context, next: Next| {
    println!("🔐 API route - checking auth...");
    next.run(ctx).await
});
```

### Route Builder Pattern

```rust
app.route("/users")
    .get(async |c: Context| {
        Ok(c.json(&serde_json::json!({
            "users": [...]
        })))
    })
    .post(async |c: Context| {
        Ok(c.json(&serde_json::json!({
            "message": "User created"
        })))
    });
```

### Path Parameters

```rust
app.route("/users/:id")
    .get(async |c: Context| {
        let id = c.req.param("id")?;
        Ok(c.json(&serde_json::json!({
            "id": id,
            "name": "John Doe"
        })))
    });
```

## Dependencies

- **ruxno** - Core web framework
- **ruxno-middleware** - Middleware collection (rate-limit, pretty-json)
- **tokio** - Async runtime
- **serde** / **serde_json** - JSON serialization
- **tabled** - Beautiful table formatting
- **reqwest** - HTTP client for testing

## Next Steps

- Explore the [middleware patterns example](../02_middleware_patterns)
- Check out the [ruxno documentation](../../ruxno/README.md)
- Try adding your own middleware
- Implement real authentication
