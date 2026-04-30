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
                title { "Hello, world!" }
                script src="https://cdn.tailwindcss.com" {}
            }
            body class="bg-gray-50" {
                div class="container mx-auto px-4 py-8" {
                    h1 class="text-4xl font-bold text-gray-800 mb-6" { "Hello, world!" }
                    div class="bg-white rounded-lg shadow-md p-6 mb-6" {
                        p class="text-lg mb-2" { "Welcome to the Ruxno HTTP Sniffer Example!" }
                        p class="text-gray-600" { "This server demonstrates HTTP request sniffing similar to Node.js utilities." }
                    }
                    div class="space-y-3" {
                        a href="/osinfo" class="block text-blue-600 hover:text-blue-800 font-medium" { "→ OS Info" }
                        a href="/users" class="block text-blue-600 hover:text-blue-800 font-medium" { "→ User API" }
                        a href="/api/status" class="block text-blue-600 hover:text-blue-800 font-medium" { "→ API Status" }
                        a href="/admin" class="block text-blue-600 hover:text-blue-800 font-medium" { "→ Admin Dashboard" }
                        a href="/test-html" class="block text-blue-600 hover:text-blue-800 font-medium" { "→ Test Ruxno HTML Macro" }
                    }
                }
            }
        }
    };

    Ok(ctx.html(markup.into_string()))
}

/// Test handler for ruxno-html macro
pub async fn test_html_macro(ctx: Context<AppEnv>) -> Result<Response, CoreError> {
    let name = "Ruxno";
    let version = "0.1.0";
    let items = ["Fast", "Type-safe", "Ergonomic"];

    let markup = ruxno_html::html! {
        <html>
            <head>
                <meta charset="UTF-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1.0" />
                <title>Test Ruxno HTML Macro</title>
                <script src="https://cdn.tailwindcss.com"></script>
            </head>
            <body class="bg-gray-50">
                <div class="container mx-auto px-4 py-8">
                    <h1 class="text-4xl font-bold text-gray-800 mb-6">Testing {name} HTML Macro</h1>
                    <div class="bg-white rounded-lg shadow-md p-6 mb-6">
                        <p class="text-lg mb-2"><span class="font-semibold">Version:</span> {version}</p>
                        <p class="text-gray-600">This page is rendered using the ruxno-html macro!</p>
                    </div>
                    <h2 class="text-2xl font-semibold text-gray-800 mb-4">Features:</h2>
                    <ul class="list-disc list-inside space-y-2 mb-6">
                        <li class="text-gray-700">{items[0]}</li>
                        <li class="text-gray-700">{items[1]}</li>
                        <li class="text-gray-700">{items[2]}</li>
                    </ul>
                    <a href="/" class="inline-block bg-blue-600 hover:bg-blue-700 text-white font-medium px-6 py-2 rounded-lg transition-colors">Back to Home</a>
                </div>
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
    let available_memory = sys.available_memory();
    let used_memory = sys.used_memory();

    // Get CPU information
    let cpus = sys.cpus();
    let cpu_info = cpus
        .iter()
        .enumerate()
        .map(|(i, cpu)| format!("CPU {}: {} - {}%", i, cpu.brand(), cpu.cpu_usage()))
        .collect::<Vec<_>>()
        .join("<br>");

    // Get network interfaces
    let networks = Networks::new_with_refreshed_list();
    let network_info = networks
        .iter()
        .map(|(name, data)| {
            format!(
                "{}: received {} bytes, transmitted {} bytes",
                name,
                data.total_received(),
                data.total_transmitted()
            )
        })
        .collect::<Vec<_>>()
        .join("<br>");

    // Get disk information
    let disks = Disks::new_with_refreshed_list();
    let disk_info = disks
        .iter()
        .map(|disk| {
            format!(
                "{}: {} / {} ({} available)",
                disk.mount_point().display(),
                format_bytes(disk.total_space() - disk.available_space()),
                format_bytes(disk.total_space()),
                format_bytes(disk.available_space())
            )
        })
        .collect::<Vec<_>>()
        .join("<br>");

    // Format uptime
    let uptime_formatted = format_uptime(uptime);

    let markup = html! {
        html {
            head {
                meta charset="UTF-8";
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                title { "Operating System Info" }
                script src="https://cdn.tailwindcss.com" {}
            }
            body class="bg-gray-50" {
                div class="container mx-auto px-4 py-8" {
                    h1 class="text-4xl font-bold text-gray-800 mb-6" { "Operating System Info" }
                    div class="bg-white rounded-lg shadow-md overflow-hidden" {
                        table class="min-w-full divide-y divide-gray-200" {
                            tbody class="bg-white divide-y divide-gray-200" {
                                tr {
                                    th class="px-6 py-4 text-left text-sm font-semibold text-gray-900 bg-gray-50" { "Host Name" }
                                    td class="px-6 py-4 text-sm text-gray-700" { (hostname) }
                                }
                                tr {
                                    th class="px-6 py-4 text-left text-sm font-semibold text-gray-900 bg-gray-50" { "OS Type" }
                                    td class="px-6 py-4 text-sm text-gray-700" { (os_name) " " (os_version) }
                                }
                                tr {
                                    th class="px-6 py-4 text-left text-sm font-semibold text-gray-900 bg-gray-50" { "Kernel Version" }
                                    td class="px-6 py-4 text-sm text-gray-700" { (kernel_version) }
                                }
                                tr {
                                    th class="px-6 py-4 text-left text-sm font-semibold text-gray-900 bg-gray-50" { "Uptime" }
                                    td class="px-6 py-4 text-sm text-gray-700" { (uptime_formatted) }
                                }
                                tr {
                                    th class="px-6 py-4 text-left text-sm font-semibold text-gray-900 bg-gray-50" { "Memory" }
                                    td class="px-6 py-4 text-sm text-gray-700" {
                                        div { "Total: " (format_bytes(total_memory)) }
                                        div { "Used: " (format_bytes(used_memory)) }
                                        div { "Available: " (format_bytes(available_memory)) }
                                    }
                                }
                                tr {
                                    th class="px-6 py-4 text-left text-sm font-semibold text-gray-900 bg-gray-50" { "CPU's" }
                                    td class="px-6 py-4 text-sm text-gray-700" {
                                        pre class="bg-gray-50 p-3 rounded text-xs overflow-x-auto" { (PreEscaped(&cpu_info)) }
                                    }
                                }
                                tr {
                                    th class="px-6 py-4 text-left text-sm font-semibold text-gray-900 bg-gray-50" { "Network Interfaces" }
                                    td class="px-6 py-4 text-sm text-gray-700" {
                                        pre class="bg-gray-50 p-3 rounded text-xs overflow-x-auto" {
                                            @if network_info.is_empty() {
                                                "No network interfaces found"
                                            } @else {
                                                (PreEscaped(&network_info))
                                            }
                                        }
                                    }
                                }
                                tr {
                                    th class="px-6 py-4 text-left text-sm font-semibold text-gray-900 bg-gray-50" { "Disks" }
                                    td class="px-6 py-4 text-sm text-gray-700" {
                                        pre class="bg-gray-50 p-3 rounded text-xs overflow-x-auto" {
                                            @if disk_info.is_empty() {
                                                "No disk information available"
                                            } @else {
                                                (PreEscaped(disk_info))
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    div class="mt-6" {
                        a href="/" class="inline-block bg-blue-600 hover:bg-blue-700 text-white font-medium px-6 py-2 rounded-lg transition-colors" { "← Back to Home" }
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
