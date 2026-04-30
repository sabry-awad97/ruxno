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

/// Middleware information with phase
#[derive(Tabled)]
pub struct MiddlewareInfo {
    #[tabled(rename = "Middleware")]
    pub name: &'static str,
    #[tabled(rename = "Phase")]
    pub phase: &'static str,
    #[tabled(rename = "Pattern")]
    pub pattern: &'static str,
}

/// Print server startup information with nice tables
pub fn print_server_info() {
    println!("🚀 Server starting on http://127.0.0.1:3000");
    println!();

    print_features();
    print_middleware();
    print_routes();

    println!("💡 Tip: All JSON responses are automatically pretty-printed!");
    println!();
}

/// Print features table
fn print_features() {
    let features = vec![
        Feature {
            name: "Unified Middleware System",
            status: "✓ Enabled (Pre/Post routing)",
        },
        Feature {
            name: "Pretty JSON responses",
            status: "✓ Enabled (2-space)",
        },
        Feature {
            name: "HTTP Request Sniffer",
            status: "✓ Enabled",
        },
        Feature {
            name: "CORS",
            status: "✓ Enabled (permissive)",
        },
        Feature {
            name: "Rate limiting",
            status: "○ Not configured",
        },
    ];

    println!("✨ Features:");
    let mut table = Table::new(features);
    table.with(Style::rounded());
    println!("{}", table);
    println!();
}

/// Print middleware table with phases
fn print_middleware() {
    let middleware = vec![
        MiddlewareInfo {
            name: "CORS",
            phase: "Pre-Routing",
            pattern: "*",
        },
        MiddlewareInfo {
            name: "HTTP Sniffer",
            phase: "Post-Routing",
            pattern: "*",
        },
        MiddlewareInfo {
            name: "Logger",
            phase: "Post-Routing",
            pattern: "*",
        },
        MiddlewareInfo {
            name: "Pretty JSON",
            phase: "Post-Routing",
            pattern: "*",
        },
        MiddlewareInfo {
            name: "API Auth",
            phase: "Post-Routing",
            pattern: "/api/*",
        },
        MiddlewareInfo {
            name: "Admin Auth",
            phase: "Post-Routing",
            pattern: "/admin",
        },
    ];

    println!("🔧 Middleware (Execution Order):");
    let mut table = Table::new(middleware);
    table.with(Style::rounded());
    println!("{}", table);
    println!();
    println!("   Pre-Routing:  Runs BEFORE route matching (no route params)");
    println!("   Post-Routing: Runs AFTER route matching (has route params)");
    println!();
}

/// Print routes table
fn print_routes() {
    let routes = vec![
        Route {
            method: "GET",
            path: "/",
            middleware: "Post-Routing (Global)",
        },
        Route {
            method: "GET",
            path: "/osinfo",
            middleware: "Post-Routing (Global)",
        },
        Route {
            method: "GET",
            path: "/users",
            middleware: "Post-Routing (Global)",
        },
        Route {
            method: "POST",
            path: "/users",
            middleware: "Post-Routing (Global)",
        },
        Route {
            method: "GET",
            path: "/users/:id",
            middleware: "Post-Routing (Global)",
        },
        Route {
            method: "PUT",
            path: "/users/:id",
            middleware: "Post-Routing (Global)",
        },
        Route {
            method: "DELETE",
            path: "/users/:id",
            middleware: "Post-Routing (Global)",
        },
        Route {
            method: "GET",
            path: "/admin",
            middleware: "Post-Routing (Global + Admin)",
        },
        Route {
            method: "POST",
            path: "/admin",
            middleware: "Post-Routing (Global + Admin)",
        },
        Route {
            method: "GET",
            path: "/api/status",
            middleware: "Post-Routing (Global + API)",
        },
    ];

    println!("📍 Routes:");
    let mut table = Table::new(routes);
    table.with(Style::rounded());
    println!("{}", table);
    println!();
}
