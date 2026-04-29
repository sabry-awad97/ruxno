//! Test client for the hello world example
//!
//! This binary tests all endpoints using reqwest with async/await.
//!
//! Run the server first:
//! ```bash
//! cargo run
//! ```
//!
//! Then run this test client in another terminal:
//! ```bash
//! cargo run --bin test_client
//! ```

use reqwest::Client;
use serde_json::Value;
use std::time::Duration;

const BASE_URL: &str = "http://127.0.0.1:3000";

#[tokio::main]
async fn main() {
    println!("🧪 Testing Ruxno Hello World Example");
    println!("=====================================");
    println!();

    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .expect("Failed to create HTTP client");

    // Test 1: GET /
    test_endpoint(&client, "GET", "/", None, "Home endpoint").await;

    // Test 2: GET /users
    test_endpoint(&client, "GET", "/users", None, "List users").await;

    // Test 3: POST /users
    test_endpoint(
        &client,
        "POST",
        "/users",
        Some(serde_json::json!({
            "name": "Test User",
            "email": "test@example.com"
        })),
        "Create user",
    )
    .await;

    // Test 4: GET /users/:id
    test_endpoint(&client, "GET", "/users/123", None, "Get user by ID").await;

    // Test 5: PUT /users/:id
    test_endpoint(
        &client,
        "PUT",
        "/users/123",
        Some(serde_json::json!({
            "name": "Updated User",
            "email": "updated@example.com"
        })),
        "Update user",
    )
    .await;

    // Test 6: DELETE /users/:id
    test_endpoint(&client, "DELETE", "/users/456", None, "Delete user").await;

    // Test 7: GET /admin
    test_endpoint(&client, "GET", "/admin", None, "Admin dashboard").await;

    // Test 8: POST /admin
    test_endpoint(
        &client,
        "POST",
        "/admin",
        Some(serde_json::json!({
            "action": "update_settings",
            "settings": {
                "maintenance_mode": false
            }
        })),
        "Admin action",
    )
    .await;

    // Test 9: GET /api/status
    test_endpoint(&client, "GET", "/api/status", None, "API status").await;

    println!();
    println!("✅ All tests completed!");
}

async fn test_endpoint(
    client: &Client,
    method: &str,
    path: &str,
    body: Option<Value>,
    description: &str,
) {
    println!("📍 Testing: {} {} - {}", method, path, description);
    println!("   URL: {}{}", BASE_URL, path);

    let url = format!("{}{}", BASE_URL, path);

    let response = match method {
        "GET" => client.get(&url).send().await,
        "POST" => {
            let mut req = client.post(&url);
            if let Some(json) = body {
                req = req.json(&json);
            }
            req.send().await
        }
        "PUT" => {
            let mut req = client.put(&url);
            if let Some(json) = body {
                req = req.json(&json);
            }
            req.send().await
        }
        "DELETE" => client.delete(&url).send().await,
        _ => panic!("Unsupported method: {}", method),
    };

    match response {
        Ok(resp) => {
            let status = resp.status();
            println!("   Status: {}", status);

            if let Ok(text) = resp.text().await {
                // Try to parse as JSON and pretty print
                if let Ok(json) = serde_json::from_str::<Value>(&text) {
                    println!("   Response:");
                    let pretty = serde_json::to_string_pretty(&json).unwrap();
                    for line in pretty.lines() {
                        println!("   {}", line);
                    }
                } else {
                    println!("   Response: {}", text);
                }
            }

            if status.is_success() {
                println!("   ✅ Success");
            } else {
                println!("   ⚠️  Non-success status");
            }
        }
        Err(e) => {
            println!("   ❌ Error: {}", e);
        }
    }

    println!();
}
