//! Home Controller
//!
//! Handles home page and general application routes.
//! Provides HTML responses similar to the Node.js HTTP sniffer example.

use crate::config::AppEnv;
use maud::{PreEscaped, html};
use ruxno::core::CoreError;
use ruxno::prelude::*;
use sysinfo::{Disks, Networks, System};

/// Home page handler - returns HTML page
pub async fn index(ctx: Context<AppEnv>) -> Result<Response, CoreError> {
    let markup: maud::PreEscaped<String> = html! {
        html {
            head {
                meta charset="UTF-8";
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                title { "Ruxno Framework" }
                script src="https://cdn.tailwindcss.com" {}
                style {
                    (PreEscaped(r#"
                        @import url('https://fonts.googleapis.com/css2?family=Space+Grotesk:wght@400;500;600;700&family=JetBrains+Mono:wght@400;500;600&display=swap');
                        
                        * {
                            margin: 0;
                            padding: 0;
                            box-sizing: border-box;
                        }
                        
                        body {
                            font-family: 'Space Grotesk', -apple-system, BlinkMacSystemFont, sans-serif;
                            background: linear-gradient(135deg, #0a0e27 0%, #1a1f3a 50%, #0f1419 100%);
                            color: #e4e4e7;
                            min-height: 100vh;
                            overflow-x: hidden;
                        }
                        
                        .mono {
                            font-family: 'JetBrains Mono', 'Courier New', monospace;
                        }
                        
                        .gradient-text {
                            background: linear-gradient(135deg, #00d4ff 0%, #00ff88 100%);
                            -webkit-background-clip: text;
                            -webkit-text-fill-color: transparent;
                            background-clip: text;
                        }
                        
                        .glow-border {
                            position: relative;
                            background: rgba(15, 20, 35, 0.6);
                            backdrop-filter: blur(10px);
                            border: 1px solid rgba(0, 212, 255, 0.2);
                            transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
                        }
                        
                        .glow-border::before {
                            content: '';
                            position: absolute;
                            inset: -1px;
                            background: linear-gradient(135deg, #00d4ff, #00ff88);
                            border-radius: inherit;
                            opacity: 0;
                            transition: opacity 0.3s ease;
                            z-index: -1;
                        }
                        
                        .glow-border:hover {
                            transform: translateY(-2px);
                            border-color: rgba(0, 212, 255, 0.5);
                            box-shadow: 0 8px 32px rgba(0, 212, 255, 0.15);
                        }
                        
                        .glow-border:hover::before {
                            opacity: 0.1;
                        }
                        
                        .nav-card {
                            position: relative;
                            overflow: hidden;
                        }
                        
                        .nav-card::after {
                            content: '';
                            position: absolute;
                            top: 0;
                            left: -100%;
                            width: 100%;
                            height: 100%;
                            background: linear-gradient(90deg, transparent, rgba(0, 212, 255, 0.1), transparent);
                            transition: left 0.5s ease;
                        }
                        
                        .nav-card:hover::after {
                            left: 100%;
                        }
                        
                        .pulse-dot {
                            animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
                        }
                        
                        @keyframes pulse {
                            0%, 100% {
                                opacity: 1;
                            }
                            50% {
                                opacity: 0.5;
                            }
                        }
                        
                        .grid-pattern {
                            background-image: 
                                linear-gradient(rgba(0, 212, 255, 0.03) 1px, transparent 1px),
                                linear-gradient(90deg, rgba(0, 212, 255, 0.03) 1px, transparent 1px);
                            background-size: 50px 50px;
                        }
                        
                        .feature-badge {
                            display: inline-flex;
                            align-items: center;
                            gap: 0.5rem;
                            padding: 0.375rem 0.875rem;
                            background: rgba(0, 212, 255, 0.1);
                            border: 1px solid rgba(0, 212, 255, 0.3);
                            border-radius: 9999px;
                            font-size: 0.75rem;
                            font-weight: 500;
                            color: #00d4ff;
                            transition: all 0.2s ease;
                        }
                        
                        .feature-badge:hover {
                            background: rgba(0, 212, 255, 0.2);
                            transform: scale(1.05);
                        }
                        
                        @media (max-width: 768px) {
                            .hero-title {
                                font-size: 3rem !important;
                            }
                        }
                    "#))
                }
            }
            body class="grid-pattern" {
                // Header with status indicator
                header class="border-b border-gray-800/50 backdrop-blur-sm sticky top-0 z-50" style="background: rgba(10, 14, 39, 0.8);" {
                    div class="max-w-7xl mx-auto px-6 py-4 flex items-center justify-between" {
                        div class="flex items-center gap-3" {
                            div class="w-2 h-2 rounded-full bg-green-400 pulse-dot" {}
                            span class="mono text-sm text-gray-400" { "SYSTEM ONLINE" }
                        }
                        div class="flex items-center gap-4" {
                            span class="mono text-xs text-gray-500" { "v0.1.0" }
                            span class="mono text-xs text-cyan-400" { "RUXNO" }
                        }
                    }
                }

                // Main content
                main class="max-w-7xl mx-auto px-6 py-16" {
                    // Hero section with massive typography
                    div class="mb-20" {
                        div class="flex items-center gap-3 mb-6" {
                            span class="feature-badge" {
                                span { "⚡" }
                                span { "BLAZING FAST" }
                            }
                            span class="feature-badge" {
                                span { "🔒" }
                                span { "TYPE SAFE" }
                            }
                            span class="feature-badge" {
                                span { "🦀" }
                                span { "RUST POWERED" }
                            }
                        }

                        h1 class="hero-title text-7xl md:text-8xl font-bold mb-6 leading-none" {
                            span class="text-white" { "Build " }
                            span class="gradient-text" { "faster" }
                            br;
                            span class="text-white" { "with " }
                            span class="mono gradient-text" { "Ruxno" }
                        }

                        p class="text-xl text-gray-400 max-w-2xl leading-relaxed" {
                            "A modern Rust web framework that combines "
                            span class="text-cyan-400 font-semibold" { "blazing performance" }
                            " with "
                            span class="text-cyan-400 font-semibold" { "developer ergonomics" }
                            ". Built for production, designed for speed."
                        }
                    }

                    // Asymmetric navigation grid
                    div class="grid grid-cols-1 md:grid-cols-12 gap-4 mb-12" {
                        // Large featured card
                        a href="/osinfo" class="md:col-span-7 md:row-span-2 glow-border nav-card rounded-2xl p-8 group" {
                            div class="flex flex-col h-full justify-between" {
                                div {
                                    div class="inline-flex items-center gap-2 mb-4 text-cyan-400" {
                                        span class="text-2xl" { "💻" }
                                        span class="mono text-xs font-semibold tracking-wider" { "SYSTEM" }
                                    }
                                    h2 class="text-3xl font-bold mb-3 text-white group-hover:text-cyan-400 transition-colors" {
                                        "OS Information"
                                    }
                                    p class="text-gray-400 text-sm leading-relaxed" {
                                        "Real-time system metrics, CPU usage, memory stats, and network interfaces. "
                                        "Monitor your infrastructure at a glance."
                                    }
                                }
                                div class="flex items-center gap-2 text-cyan-400 mono text-sm font-semibold" {
                                    span { "EXPLORE" }
                                    span class="group-hover:translate-x-1 transition-transform" { "→" }
                                }
                            }
                        }

                        // Top right card
                        a href="/users" class="md:col-span-5 glow-border nav-card rounded-2xl p-6 group" {
                            div class="flex items-start justify-between" {
                                div {
                                    span class="text-2xl mb-3 block" { "👥" }
                                    h3 class="text-xl font-bold text-white group-hover:text-cyan-400 transition-colors mb-2" {
                                        "User API"
                                    }
                                    p class="text-gray-400 text-sm" {
                                        "RESTful user management endpoints"
                                    }
                                }
                                span class="text-cyan-400 text-2xl group-hover:translate-x-1 transition-transform" { "→" }
                            }
                        }

                        // Middle right card
                        a href="/api/status" class="md:col-span-5 glow-border nav-card rounded-2xl p-6 group" {
                            div class="flex items-start justify-between" {
                                div {
                                    span class="text-2xl mb-3 block" { "📊" }
                                    h3 class="text-xl font-bold text-white group-hover:text-cyan-400 transition-colors mb-2" {
                                        "API Status"
                                    }
                                    p class="text-gray-400 text-sm" {
                                        "Health checks and system diagnostics"
                                    }
                                }
                                span class="text-cyan-400 text-2xl group-hover:translate-x-1 transition-transform" { "→" }
                            }
                        }

                        // Bottom left card
                        a href="/admin" class="md:col-span-5 glow-border nav-card rounded-2xl p-6 group" {
                            div class="flex items-start justify-between" {
                                div {
                                    span class="text-2xl mb-3 block" { "⚙️" }
                                    h3 class="text-xl font-bold text-white group-hover:text-cyan-400 transition-colors mb-2" {
                                        "Admin Dashboard"
                                    }
                                    p class="text-gray-400 text-sm" {
                                        "Control panel and configuration"
                                    }
                                }
                                span class="text-cyan-400 text-2xl group-hover:translate-x-1 transition-transform" { "→" }
                            }
                        }

                        // Bottom right card
                        a href="/test-html" class="md:col-span-7 glow-border nav-card rounded-2xl p-6 group" {
                            div class="flex items-start justify-between" {
                                div {
                                    span class="text-2xl mb-3 block" { "🧪" }
                                    h3 class="text-xl font-bold text-white group-hover:text-cyan-400 transition-colors mb-2" {
                                        "Test Ruxno HTML Macro"
                                    }
                                    p class="text-gray-400 text-sm" {
                                        "Explore the type-safe HTML templating system with compile-time guarantees"
                                    }
                                }
                                span class="text-cyan-400 text-2xl group-hover:translate-x-1 transition-transform" { "→" }
                            }
                        }
                    }

                    // Stats section
                    div class="grid grid-cols-2 md:grid-cols-4 gap-4" {
                        div class="glow-border rounded-xl p-6 text-center" {
                            div class="mono text-3xl font-bold gradient-text mb-2" { "<1ms" }
                            div class="text-sm text-gray-400" { "Response Time" }
                        }
                        div class="glow-border rounded-xl p-6 text-center" {
                            div class="mono text-3xl font-bold gradient-text mb-2" { "100%" }
                            div class="text-sm text-gray-400" { "Type Safe" }
                        }
                        div class="glow-border rounded-xl p-6 text-center" {
                            div class="mono text-3xl font-bold gradient-text mb-2" { "0" }
                            div class="text-sm text-gray-400" { "Runtime Errors" }
                        }
                        div class="glow-border rounded-xl p-6 text-center" {
                            div class="mono text-3xl font-bold gradient-text mb-2" { "∞" }
                            div class="text-sm text-gray-400" { "Possibilities" }
                        }
                    }
                }

                // Footer
                footer class="border-t border-gray-800/50 mt-20" {
                    div class="max-w-7xl mx-auto px-6 py-8" {
                        div class="flex flex-col md:flex-row items-center justify-between gap-4" {
                            p class="mono text-sm text-gray-500" {
                                "Built with "
                                span class="text-cyan-400" { "♥" }
                                " using Rust"
                            }
                            div class="flex items-center gap-6 mono text-xs text-gray-500" {
                                span { "© 2024 Ruxno" }
                                span { "•" }
                                span { "MIT License" }
                            }
                        }
                    }
                }
            }
        }
    };

    Ok(ctx.html(markup.into_string()))
}

/// Test handler for ruxno-html macro
pub async fn test_html_macro(ctx: Context<AppEnv>) -> Result<Response, CoreError> {
    let name = "Ruxno HTML";
    let version = "0.1.0";
    let features = [
        (
            "⚡",
            "Blazing Fast",
            "Compile-time HTML generation with zero runtime overhead",
        ),
        (
            "🔒",
            "Type Safe",
            "Catch HTML errors at compile time, not in production",
        ),
        (
            "🎨",
            "Ergonomic",
            "JSX-like syntax that feels natural and intuitive",
        ),
        (
            "🦀",
            "Rust Native",
            "Seamless integration with Rust's type system",
        ),
        (
            "📦",
            "Zero Cost",
            "No runtime dependencies or performance penalties",
        ),
        (
            "✨",
            "Modern",
            "Support for dynamic content and conditional rendering",
        ),
    ];

    let code_examples = [
        (
            "Basic Syntax",
            r#"html! {
    <div class="container">
        <h1>Hello {name}</h1>
    </div>
}"#,
        ),
        (
            "Conditionals",
            r#"html! {
    <div>
        {if is_admin {
            <button>Admin Panel</button>
        }}
    </div>
}"#,
        ),
        (
            "Loops",
            r#"html! {
    <ul>
        {for item in items {
            <li>{item}</li>
        }}
    </ul>
}"#,
        ),
    ];

    let markup = ruxno_html::html! {
        <html>
            <head>
                <meta charset="UTF-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1.0" />
                <title>Ruxno HTML Macro - Type-Safe Templating</title>
                <script src="https://cdn.tailwindcss.com"></script>
                <style>
                    {"@import url('https://fonts.googleapis.com/css2?family=Space+Grotesk:wght@400;500;600;700&family=JetBrains+Mono:wght@400;500;600&display=swap');"}

                    {"* { margin: 0; padding: 0; box-sizing: border-box; }"}

                    {"body {
                        font-family: 'Space Grotesk', -apple-system, BlinkMacSystemFont, sans-serif;
                        background: linear-gradient(135deg, #0a0e27 0%, #1a1f3a 50%, #0f1419 100%);
                        color: #e4e4e7;
                        min-height: 100vh;
                    }"}

                    {".mono { font-family: 'JetBrains Mono', 'Courier New', monospace; }"}

                    {".gradient-text {
                        background: linear-gradient(135deg, #00d4ff 0%, #00ff88 100%);
                        -webkit-background-clip: text;
                        -webkit-text-fill-color: transparent;
                        background-clip: text;
                    }"}

                    {".card {
                        background: rgba(15, 20, 35, 0.6);
                        backdrop-filter: blur(10px);
                        border: 1px solid rgba(0, 212, 255, 0.2);
                        border-radius: 1rem;
                        padding: 1.5rem;
                        transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
                    }"}

                    {".card:hover {
                        border-color: rgba(0, 212, 255, 0.4);
                        transform: translateY(-2px);
                        box-shadow: 0 8px 32px rgba(0, 212, 255, 0.1);
                    }"}

                    {".code-block {
                        background: rgba(0, 0, 0, 0.4);
                        border: 1px solid rgba(0, 212, 255, 0.2);
                        border-radius: 0.5rem;
                        padding: 1rem;
                        overflow-x: auto;
                        font-size: 0.875rem;
                        line-height: 1.5;
                    }"}

                    {".pulse-dot {
                        animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
                    }"}

                    {"@keyframes pulse {
                        0%, 100% { opacity: 1; }
                        50% { opacity: 0.5; }
                    }"}

                    {".grid-pattern {
                        background-image: 
                            linear-gradient(rgba(0, 212, 255, 0.03) 1px, transparent 1px),
                            linear-gradient(90deg, rgba(0, 212, 255, 0.03) 1px, transparent 1px);
                        background-size: 50px 50px;
                    }"}

                    {".feature-icon {
                        font-size: 2.5rem;
                        filter: drop-shadow(0 0 10px rgba(0, 212, 255, 0.3));
                    }"}
                </style>
            </head>
            <body class="grid-pattern">
                <header class="border-b border-gray-800/50 backdrop-blur-sm sticky top-0 z-50" style="background: rgba(10, 14, 39, 0.8);">
                    <div class="max-w-7xl mx-auto px-6 py-4 flex items-center justify-between">
                        <a href="/" class="flex items-center gap-3 hover:opacity-80 transition-opacity">
                            <span class="text-2xl">{"←"}</span>
                            <span class="mono text-sm text-gray-400">{"BACK TO HOME"}</span>
                        </a>
                        <div class="flex items-center gap-3">
                            <div class="w-2 h-2 rounded-full bg-green-400 pulse-dot"></div>
                            <span class="mono text-sm text-gray-400">{"MACRO DEMO"}</span>
                        </div>
                    </div>
                </header>

                <main class="max-w-7xl mx-auto px-6 py-12">
                    <div class="mb-12">
                        <div class="inline-flex items-center gap-2 px-4 py-2 rounded-full mb-6"
                             style="background: rgba(0, 212, 255, 0.1); border: 1px solid rgba(0, 212, 255, 0.3);">
                            <span class="text-cyan-400 text-sm font-semibold">{"✨ COMPILE-TIME MAGIC"}</span>
                        </div>

                        <h1 class="text-5xl md:text-6xl font-bold mb-4">
                            <span class="gradient-text">{name}</span>
                        </h1>

                        <p class="text-xl text-gray-400 max-w-3xl leading-relaxed mb-6">
                            {"Type-safe HTML templating with "}
                            <span class="text-cyan-400 font-semibold">{"compile-time guarantees"}</span>
                            {". Write JSX-like syntax, get blazing fast Rust code."}
                        </p>

                        <div class="flex items-center gap-4">
                            <span class="mono text-sm text-gray-500">{"Version"}</span>
                            <span class="mono text-lg font-bold text-cyan-400">{version}</span>
                        </div>
                    </div>

                    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 mb-12">
                        {
                            features.iter().map(|(icon, title, description)| {
                                format!(
                                    r#"<div class="card">
                                        <div class="feature-icon mb-4">{}</div>
                                        <h3 class="text-xl font-bold text-white mb-2">{}</h3>
                                        <p class="text-gray-400 text-sm leading-relaxed">{}</p>
                                    </div>"#,
                                    icon, title, description
                                )
                            }).collect::<Vec<_>>().join("")
                        }
                    </div>

                    <div class="mb-12">
                        <h2 class="text-3xl font-bold text-white mb-6 flex items-center gap-3">
                            <span>{"🧪"}</span>
                            <span>{"Code Examples"}</span>
                        </h2>

                        <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                            {
                                code_examples.iter().map(|(title, code)| {
                                    format!(
                                        r#"<div class="card">
                                            <h3 class="text-lg font-bold text-cyan-400 mb-4 mono">{}</h3>
                                            <pre class="code-block mono text-gray-300"><code>{}</code></pre>
                                        </div>"#,
                                        title, code
                                    )
                                }).collect::<Vec<_>>().join("")
                            }
                        </div>
                    </div>

                    <div class="card mb-12">
                        <h2 class="text-3xl font-bold text-white mb-6 flex items-center gap-3">
                            <span>{"🎯"}</span>
                            <span>{"Why Ruxno HTML?"}</span>
                        </h2>

                        <div class="grid grid-cols-1 md:grid-cols-2 gap-8">
                            <div>
                                <h3 class="text-xl font-semibold text-cyan-400 mb-3">{"Traditional Templating"}</h3>
                                <ul class="space-y-2 text-gray-400">
                                    <li class="flex items-start gap-2">
                                        <span class="text-red-400">{"❌"}</span>
                                        <span>{"Runtime string concatenation"}</span>
                                    </li>
                                    <li class="flex items-start gap-2">
                                        <span class="text-red-400">{"❌"}</span>
                                        <span>{"No compile-time validation"}</span>
                                    </li>
                                    <li class="flex items-start gap-2">
                                        <span class="text-red-400">{"❌"}</span>
                                        <span>{"XSS vulnerabilities"}</span>
                                    </li>
                                    <li class="flex items-start gap-2">
                                        <span class="text-red-400">{"❌"}</span>
                                        <span>{"Performance overhead"}</span>
                                    </li>
                                </ul>
                            </div>

                            <div>
                                <h3 class="text-xl font-semibold text-cyan-400 mb-3">{"Ruxno HTML Macro"}</h3>
                                <ul class="space-y-2 text-gray-400">
                                    <li class="flex items-start gap-2">
                                        <span class="text-green-400">{"✅"}</span>
                                        <span>{"Compile-time code generation"}</span>
                                    </li>
                                    <li class="flex items-start gap-2">
                                        <span class="text-green-400">{"✅"}</span>
                                        <span>{"Type-safe at compile time"}</span>
                                    </li>
                                    <li class="flex items-start gap-2">
                                        <span class="text-green-400">{"✅"}</span>
                                        <span>{"Automatic escaping by default"}</span>
                                    </li>
                                    <li class="flex items-start gap-2">
                                        <span class="text-green-400">{"✅"}</span>
                                        <span>{"Zero runtime overhead"}</span>
                                    </li>
                                </ul>
                            </div>
                        </div>
                    </div>

                    <div class="card text-center">
                        <div class="text-6xl mb-4">{"🚀"}</div>
                        <h2 class="text-2xl font-bold text-white mb-3">{"Ready to Build?"}</h2>
                        <p class="text-gray-400 mb-6 max-w-2xl mx-auto">
                            {"This entire page was rendered using the ruxno-html macro with compile-time guarantees. "}
                            {"No runtime templating, no string concatenation, just pure Rust performance."}
                        </p>
                        <a href="/"
                           class="inline-block px-8 py-3 rounded-lg font-semibold transition-all"
                           style="background: linear-gradient(135deg, #00d4ff, #00ff88); color: #0a0e27;">
                            {"Explore More Examples"}
                        </a>
                    </div>
                </main>

                <footer class="border-t border-gray-800/50 mt-12">
                    <div class="max-w-7xl mx-auto px-6 py-8">
                        <div class="flex items-center justify-center gap-2 mono text-sm text-gray-500">
                            <span>{"Powered by"}</span>
                            <span class="text-cyan-400">{"Ruxno"}</span>
                            <span>{"•"}</span>
                            <span>{"Type-safe HTML templating"}</span>
                        </div>
                    </div>
                </footer>
            </body>
        </html>
    };

    Ok(ctx.html(markup))
}

/// OS Info page handler - returns system information in HTML
pub async fn osinfo(ctx: Context<AppEnv>) -> Result<Response, CoreError> {
    let mut sys = System::new_all();
    sys.refresh_all();

    // Get system information using modern sysinfo API
    let hostname = System::host_name().unwrap_or_else(|| "unknown".to_string());
    let os_name = System::name().unwrap_or_else(|| "unknown".to_string());
    let os_version = System::os_version().unwrap_or_else(|| "unknown".to_string());
    let kernel_version = System::kernel_version().unwrap_or_else(|| "unknown".to_string());
    let uptime = System::uptime();
    let total_memory = sys.total_memory();
    let used_memory = sys.used_memory();
    let memory_usage_percent = ((used_memory as f64 / total_memory as f64) * 100.0) as u32;

    // Get CPU information
    let cpus = sys.cpus();
    let cpu_count = cpus.len();
    let cpu_brand = cpus.first().map(|cpu| cpu.brand()).unwrap_or("Unknown");
    let avg_cpu_usage = if !cpus.is_empty() {
        cpus.iter().map(|cpu| cpu.cpu_usage()).sum::<f32>() / cpus.len() as f32
    } else {
        0.0
    };

    // Get network interfaces
    let networks = Networks::new_with_refreshed_list();
    let network_count = networks.iter().count();
    let total_received: u64 = networks.values().map(|data| data.total_received()).sum();
    let total_transmitted: u64 = networks.values().map(|data| data.total_transmitted()).sum();

    // Get disk information
    let disks = Disks::new_with_refreshed_list();
    let disk_count = disks.iter().count();
    let total_disk_space: u64 = disks.iter().map(|disk| disk.total_space()).sum();
    let available_disk_space: u64 = disks.iter().map(|disk| disk.available_space()).sum();
    let used_disk_space = total_disk_space - available_disk_space;
    let disk_usage_percent = if total_disk_space > 0 {
        ((used_disk_space as f64 / total_disk_space as f64) * 100.0) as u32
    } else {
        0
    };

    // Format uptime
    let uptime_formatted = format_uptime(uptime);

    let markup = html! {
        html {
            head {
                meta charset="UTF-8";
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                title { "System Dashboard - Ruxno" }
                script src="https://cdn.tailwindcss.com" {}
                style {
                    (PreEscaped(r#"
                        @import url('https://fonts.googleapis.com/css2?family=Space+Grotesk:wght@400;500;600;700&family=JetBrains+Mono:wght@400;500;600&display=swap');
                        
                        * {
                            margin: 0;
                            padding: 0;
                            box-sizing: border-box;
                        }
                        
                        body {
                            font-family: 'Space Grotesk', -apple-system, BlinkMacSystemFont, sans-serif;
                            background: linear-gradient(135deg, #0a0e27 0%, #1a1f3a 50%, #0f1419 100%);
                            color: #e4e4e7;
                            min-height: 100vh;
                        }
                        
                        .mono {
                            font-family: 'JetBrains Mono', 'Courier New', monospace;
                        }
                        
                        .gradient-text {
                            background: linear-gradient(135deg, #00d4ff 0%, #00ff88 100%);
                            -webkit-background-clip: text;
                            -webkit-text-fill-color: transparent;
                            background-clip: text;
                        }
                        
                        .metric-card {
                            background: rgba(15, 20, 35, 0.6);
                            backdrop-filter: blur(10px);
                            border: 1px solid rgba(0, 212, 255, 0.2);
                            border-radius: 1rem;
                            padding: 1.5rem;
                            transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
                        }
                        
                        .metric-card:hover {
                            border-color: rgba(0, 212, 255, 0.4);
                            transform: translateY(-2px);
                            box-shadow: 0 8px 32px rgba(0, 212, 255, 0.1);
                        }
                        
                        .progress-bar {
                            width: 100%;
                            height: 8px;
                            background: rgba(0, 212, 255, 0.1);
                            border-radius: 9999px;
                            overflow: hidden;
                            position: relative;
                        }
                        
                        .progress-fill {
                            height: 100%;
                            background: linear-gradient(90deg, #00d4ff, #00ff88);
                            border-radius: 9999px;
                            transition: width 0.5s ease;
                            box-shadow: 0 0 10px rgba(0, 212, 255, 0.5);
                        }
                        
                        .data-table {
                            width: 100%;
                            border-collapse: separate;
                            border-spacing: 0;
                        }
                        
                        .data-table th {
                            background: rgba(0, 212, 255, 0.05);
                            padding: 1rem;
                            text-align: left;
                            font-weight: 600;
                            color: #00d4ff;
                            border-bottom: 1px solid rgba(0, 212, 255, 0.2);
                        }
                        
                        .data-table td {
                            padding: 1rem;
                            border-bottom: 1px solid rgba(255, 255, 255, 0.05);
                        }
                        
                        .data-table tr:last-child td {
                            border-bottom: none;
                        }
                        
                        .pulse-dot {
                            animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
                        }
                        
                        @keyframes pulse {
                            0%, 100% { opacity: 1; }
                            50% { opacity: 0.5; }
                        }
                        
                        .grid-pattern {
                            background-image: 
                                linear-gradient(rgba(0, 212, 255, 0.03) 1px, transparent 1px),
                                linear-gradient(90deg, rgba(0, 212, 255, 0.03) 1px, transparent 1px);
                            background-size: 50px 50px;
                        }
                    "#))
                }
            }
            body class="grid-pattern" {
                // Header
                header class="border-b border-gray-800/50 backdrop-blur-sm sticky top-0 z-50" style="background: rgba(10, 14, 39, 0.8);" {
                    div class="max-w-7xl mx-auto px-6 py-4 flex items-center justify-between" {
                        a href="/" class="flex items-center gap-3 hover:opacity-80 transition-opacity" {
                            span class="text-2xl" { "←" }
                            span class="mono text-sm text-gray-400" { "BACK TO HOME" }
                        }
                        div class="flex items-center gap-3" {
                            div class="w-2 h-2 rounded-full bg-green-400 pulse-dot" {}
                            span class="mono text-sm text-gray-400" { "LIVE METRICS" }
                        }
                    }
                }

                main class="max-w-7xl mx-auto px-6 py-12" {
                    // Page title
                    div class="mb-12" {
                        h1 class="text-5xl md:text-6xl font-bold mb-4" {
                            span class="gradient-text" { "System" }
                            span class="text-white" { " Dashboard" }
                        }
                        p class="text-gray-400 text-lg" {
                            "Real-time monitoring of " span class="text-cyan-400 mono" { (hostname) }
                        }
                    }

                    // Key metrics grid
                    div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 mb-8" {
                        // CPU metric
                        div class="metric-card" {
                            div class="flex items-center justify-between mb-3" {
                                span class="text-2xl" { "⚡" }
                                span class="mono text-xs text-gray-500" { "CPU" }
                            }
                            div class="mono text-3xl font-bold gradient-text mb-2" {
                                (format!("{:.1}%", avg_cpu_usage))
                            }
                            div class="text-sm text-gray-400 mb-3" {
                                (cpu_count) " cores • " (cpu_brand)
                            }
                            div class="progress-bar" {
                                div class="progress-fill" style=(format!("width: {}%", avg_cpu_usage as u32)) {}
                            }
                        }

                        // Memory metric
                        div class="metric-card" {
                            div class="flex items-center justify-between mb-3" {
                                span class="text-2xl" { "💾" }
                                span class="mono text-xs text-gray-500" { "MEMORY" }
                            }
                            div class="mono text-3xl font-bold gradient-text mb-2" {
                                (memory_usage_percent) "%"
                            }
                            div class="text-sm text-gray-400 mb-3" {
                                (format_bytes(used_memory)) " / " (format_bytes(total_memory))
                            }
                            div class="progress-bar" {
                                div class="progress-fill" style=(format!("width: {}%", memory_usage_percent)) {}
                            }
                        }

                        // Disk metric
                        div class="metric-card" {
                            div class="flex items-center justify-between mb-3" {
                                span class="text-2xl" { "💿" }
                                span class="mono text-xs text-gray-500" { "STORAGE" }
                            }
                            div class="mono text-3xl font-bold gradient-text mb-2" {
                                (disk_usage_percent) "%"
                            }
                            div class="text-sm text-gray-400 mb-3" {
                                (format_bytes(used_disk_space)) " / " (format_bytes(total_disk_space))
                            }
                            div class="progress-bar" {
                                div class="progress-fill" style=(format!("width: {}%", disk_usage_percent)) {}
                            }
                        }

                        // Uptime metric
                        div class="metric-card" {
                            div class="flex items-center justify-between mb-3" {
                                span class="text-2xl" { "⏱️" }
                                span class="mono text-xs text-gray-500" { "UPTIME" }
                            }
                            div class="mono text-2xl font-bold gradient-text mb-2" {
                                (format!("{}d", uptime / 86400))
                            }
                            div class="text-sm text-gray-400" {
                                (uptime_formatted)
                            }
                        }
                    }

                    // Detailed information sections
                    div class="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-8" {
                        // System info
                        div class="metric-card" {
                            h2 class="text-xl font-bold text-white mb-4 flex items-center gap-2" {
                                span { "🖥️" }
                                span { "System Information" }
                            }
                            table class="data-table" {
                                tbody {
                                    tr {
                                        td class="text-gray-400 mono text-sm" { "Hostname" }
                                        td class="text-white mono text-sm" { (hostname) }
                                    }
                                    tr {
                                        td class="text-gray-400 mono text-sm" { "OS" }
                                        td class="text-white mono text-sm" { (os_name) " " (os_version) }
                                    }
                                    tr {
                                        td class="text-gray-400 mono text-sm" { "Kernel" }
                                        td class="text-white mono text-sm" { (kernel_version) }
                                    }
                                    tr {
                                        td class="text-gray-400 mono text-sm" { "Architecture" }
                                        td class="text-white mono text-sm" { (std::env::consts::ARCH) }
                                    }
                                }
                            }
                        }

                        // Network info
                        div class="metric-card" {
                            h2 class="text-xl font-bold text-white mb-4 flex items-center gap-2" {
                                span { "🌐" }
                                span { "Network Activity" }
                            }
                            div class="space-y-4" {
                                div {
                                    div class="flex items-center justify-between mb-2" {
                                        span class="text-gray-400 text-sm" { "Interfaces" }
                                        span class="mono text-cyan-400 font-semibold" { (network_count) }
                                    }
                                }
                                div {
                                    div class="flex items-center justify-between mb-2" {
                                        span class="text-gray-400 text-sm" { "↓ Received" }
                                        span class="mono text-white font-semibold" { (format_bytes(total_received)) }
                                    }
                                }
                                div {
                                    div class="flex items-center justify-between mb-2" {
                                        span class="text-gray-400 text-sm" { "↑ Transmitted" }
                                        span class="mono text-white font-semibold" { (format_bytes(total_transmitted)) }
                                    }
                                }
                            }
                        }
                    }

                    // CPU details
                    div class="metric-card mb-6" {
                        h2 class="text-xl font-bold text-white mb-4 flex items-center gap-2" {
                            span { "⚙️" }
                            span { "CPU Cores" }
                        }
                        div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4" {
                            @for (i, cpu) in cpus.iter().enumerate() {
                                div class="bg-black/20 rounded-lg p-4 border border-gray-800" {
                                    div class="flex items-center justify-between mb-2" {
                                        span class="mono text-sm text-gray-400" { "Core " (i) }
                                        span class="mono text-sm font-semibold text-cyan-400" {
                                            (format!("{:.1}%", cpu.cpu_usage()))
                                        }
                                    }
                                    div class="progress-bar" {
                                        div class="progress-fill" style=(format!("width: {}%", cpu.cpu_usage() as u32)) {}
                                    }
                                }
                            }
                        }
                    }

                    // Disk details
                    @if disk_count > 0 {
                        div class="metric-card mb-6" {
                            h2 class="text-xl font-bold text-white mb-4 flex items-center gap-2" {
                                span { "💽" }
                                span { "Storage Devices" }
                            }
                            div class="space-y-4" {
                                @for disk in disks.iter() {
                                    div class="bg-black/20 rounded-lg p-4 border border-gray-800" {
                                        div class="flex items-center justify-between mb-3" {
                                            span class="mono text-sm text-white font-semibold" {
                                                (disk.mount_point().display())
                                            }
                                            span class="mono text-sm text-cyan-400" {
                                                (format_bytes(disk.available_space())) " free"
                                            }
                                        }
                                        div class="flex items-center justify-between mb-2 text-xs text-gray-400" {
                                            span {
                                                (format_bytes(disk.total_space() - disk.available_space())) " used"
                                            }
                                            span {
                                                (format_bytes(disk.total_space())) " total"
                                            }
                                        }
                                        div class="progress-bar" {
                                            @let usage = ((disk.total_space() - disk.available_space()) as f64 / disk.total_space() as f64 * 100.0) as u32;
                                            div class="progress-fill" style=(format!("width: {}%", usage)) {}
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Network interfaces details
                    @if network_count > 0 {
                        div class="metric-card" {
                            h2 class="text-xl font-bold text-white mb-4 flex items-center gap-2" {
                                span { "📡" }
                                span { "Network Interfaces" }
                            }
                            div class="space-y-3" {
                                @for (name, data) in networks.iter() {
                                    div class="bg-black/20 rounded-lg p-4 border border-gray-800" {
                                        div class="mono text-sm font-semibold text-white mb-2" { (name) }
                                        div class="grid grid-cols-2 gap-4 text-xs" {
                                            div {
                                                span class="text-gray-400" { "↓ RX: " }
                                                span class="text-cyan-400 mono" { (format_bytes(data.total_received())) }
                                            }
                                            div {
                                                span class="text-gray-400" { "↑ TX: " }
                                                span class="text-cyan-400 mono" { (format_bytes(data.total_transmitted())) }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Footer
                footer class="border-t border-gray-800/50 mt-12" {
                    div class="max-w-7xl mx-auto px-6 py-8" {
                        div class="flex items-center justify-center gap-2 mono text-sm text-gray-500" {
                            span { "Powered by" }
                            span class="text-cyan-400" { "Ruxno" }
                            span { "•" }
                            span { "Real-time system monitoring" }
                        }
                    }
                }
            }
        }
    };

    Ok(ctx.html(markup))
}

/// API status endpoint - returns JSON status information
pub async fn api_status(ctx: Context<AppEnv>) -> Result<Response, RuxnoError> {
    let env = ctx.env();
    let user_count = env.db.get_user_count().await.unwrap_or(0);

    // Get basic system info for API
    let mut sys = System::new_all();
    sys.refresh_all();

    Ok(ctx.json(&serde_json::json!({
        "status": "ok",
        "app_name": env.app_name,
        "version": env.version,
        "uptime": System::uptime(),
        "environment": "development",
        "system": {
            "hostname": System::host_name().unwrap_or_else(|| "unknown".to_string()),
            "os": System::name().unwrap_or_else(|| "unknown".to_string()),
            "memory_total": sys.total_memory(),
            "memory_available": sys.available_memory(),
            "cpu_count": sys.cpus().len()
        },
        "database": {
            "type": "in_memory",
            "user_count": user_count
        },
        "features": {
            "http_sniffer": true,
            "pretty_json": true,
            "rate_limiting": false,
            "cors": true,
            "health_check": true
        }
    })))
}

/// Format bytes into human-readable format
fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

/// Format uptime into human-readable format
fn format_uptime(uptime_seconds: u64) -> String {
    let days = uptime_seconds / 86400;
    let hours = (uptime_seconds % 86400) / 3600;
    let minutes = (uptime_seconds % 3600) / 60;
    let seconds = uptime_seconds % 60;

    if days > 0 {
        format!(
            "{} days, {} hours, {} minutes, {} seconds",
            days, hours, minutes, seconds
        )
    } else if hours > 0 {
        format!("{} hours, {} minutes, {} seconds", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{} minutes, {} seconds", minutes, seconds)
    } else {
        format!("{} seconds", seconds)
    }
}
