//! Utility functions for the hello world example

use tabled::{Table, Tabled, settings::Style};

/// Feature information
#[derive(Tabled)]
pub struct Feature {
    #[tabled(rename = "Feature")]
    pub name: &'static str,
    #[tabled(rename = "Status")]
    pub status: &'static str,
}

/// Route information
#[derive(Tabled)]
pub struct Route {
    #[tabled(rename = "Method")]
    pub method: &'static str,
    #[tabled(rename = "Path")]
    pub path: &'static str,
    #[tabled(rename = "Middleware")]
    pub middleware: &'static str,
}

/// Middleware pattern information
#[derive(Tabled)]
pub struct MiddlewarePattern {
    #[tabled(rename = "Pattern")]
    pub pattern: &'static str,
    #[tabled(rename = "Applies To")]
    pub applies_to: &'static str,
}

/// Print server startup information with nice tables
pub fn print_server_info() {
    println!("🚀 Server starting on http://127.0.0.1:3000");
    println!();

    print_features();
    print_routes();
    print_middleware_patterns();

    println!("💡 Tip: All JSON responses are automatically pretty-printed!");
    println!();
}

/// Print features table
fn print_features() {
    let features = vec![
        Feature {
            name: "Pretty JSON responses",
            status: "✓ Enabled (2-space)",
        },
        Feature {
            name: "Global middleware logging",
            status: "✓ Enabled",
        },
        Feature {
            name: "Path-specific middleware",
            status: "✓ Enabled",
        },
        Feature {
            name: "Rate limiting",
            status: "✓ Available",
        },
    ];

    println!("✨ Features:");
    let mut table = Table::new(features);
    table.with(Style::rounded());
    println!("{}", table);
    println!();
}

/// Print routes table
fn print_routes() {
    let routes = vec![
        Route {
            method: "GET",
            path: "/",
            middleware: "Global",
        },
        Route {
            method: "GET",
            path: "/users",
            middleware: "Global",
        },
        Route {
            method: "POST",
            path: "/users",
            middleware: "Global",
        },
        Route {
            method: "GET",
            path: "/admin",
            middleware: "Global + Admin",
        },
        Route {
            method: "POST",
            path: "/admin",
            middleware: "Global + Admin",
        },
        Route {
            method: "GET",
            path: "/users/:id",
            middleware: "Global",
        },
        Route {
            method: "PUT",
            path: "/users/:id",
            middleware: "Global",
        },
        Route {
            method: "DELETE",
            path: "/users/:id",
            middleware: "Global",
        },
        Route {
            method: "GET",
            path: "/api/status",
            middleware: "Global + API",
        },
    ];

    println!("📍 Routes:");
    let mut table = Table::new(routes);
    table.with(Style::rounded());
    println!("{}", table);
    println!();
}

/// Print middleware patterns table
fn print_middleware_patterns() {
    let patterns = vec![
        MiddlewarePattern {
            pattern: "*",
            applies_to: "All routes (global)",
        },
        MiddlewarePattern {
            pattern: "/api/*",
            applies_to: "API routes only",
        },
        MiddlewarePattern {
            pattern: "/admin",
            applies_to: "Admin routes only",
        },
    ];

    println!("🔧 Middleware Patterns:");
    let mut table = Table::new(patterns);
    table.with(Style::rounded());
    println!("{}", table);
    println!();
}
