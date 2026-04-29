//! Home Controller
//!
//! Handles home page and general application routes.
//! Provides HTML responses similar to the Node.js HTTP sniffer example.

use crate::config::AppEnv;
use ruxno::prelude::*;
use sysinfo::{Disks, Networks, System};

/// Home page handler - returns HTML page
pub async fn index(ctx: Context<AppEnv>) -> Result<Response, RuxnoError> {
    let html = r#"
<html>
<head>
    <title>Hello, world!</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; }
        h1 { color: #333; }
        a { color: #0066cc; text-decoration: none; }
        a:hover { text-decoration: underline; }
        .info { background: #f5f5f5; padding: 20px; border-radius: 5px; margin: 20px 0; }
    </style>
</head>
<body>
    <h1>Hello, world!</h1>
    <div class="info">
        <p>Welcome to the Ruxno HTTP Sniffer Example!</p>
        <p>This server demonstrates HTTP request sniffing similar to Node.js utilities.</p>
    </div>
    <p><a href='/osinfo'>OS Info</a></p>
    <p><a href='/users'>User API</a></p>
    <p><a href='/api/status'>API Status</a></p>
    <p><a href='/admin'>Admin Dashboard</a></p>
</body>
</html>"#;

    Ok(ctx.html(html))
}

/// OS Info page handler - returns system information in HTML
pub async fn osinfo(ctx: Context<AppEnv>) -> Result<Response, RuxnoError> {
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

    let html = format!(
        r#"
<html>
<head>
    <title>Operating System Info</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 40px; }}
        h1 {{ color: #333; }}
        table {{ border-collapse: collapse; width: 100%; margin: 20px 0; }}
        th, td {{ border: 1px solid #ddd; padding: 12px; text-align: left; }}
        th {{ background-color: #f2f2f2; font-weight: bold; }}
        pre {{ background: #f5f5f5; padding: 10px; border-radius: 3px; overflow-x: auto; }}
        .back-link {{ margin: 20px 0; }}
        a {{ color: #0066cc; text-decoration: none; }}
        a:hover {{ text-decoration: underline; }}
    </style>
</head>
<body>
    <h1>Operating System Info</h1>
    <table>
        <tr><th>Host Name</th><td>{hostname}</td></tr>
        <tr><th>OS Type</th><td>{os_name} {os_version}</td></tr>
        <tr><th>Kernel Version</th><td>{kernel_version}</td></tr>
        <tr><th>Uptime</th><td>{uptime_formatted}</td></tr>
        <tr><th>Memory</th><td>Total: {total_memory_formatted}<br>Used: {used_memory_formatted}<br>Available: {available_memory_formatted}</td></tr>
        <tr><th>CPU's</th><td><pre>{cpu_info}</pre></td></tr>
        <tr><th>Network Interfaces</th><td><pre>{network_info}</pre></td></tr>
        <tr><th>Disks</th><td><pre>{disk_info}</pre></td></tr>
    </table>
    <div class="back-link">
        <a href="/">← Back to Home</a>
    </div>
</body>
</html>"#,
        hostname = hostname,
        os_name = os_name,
        os_version = os_version,
        kernel_version = kernel_version,
        uptime_formatted = uptime_formatted,
        total_memory_formatted = format_bytes(total_memory),
        used_memory_formatted = format_bytes(used_memory),
        available_memory_formatted = format_bytes(available_memory),
        cpu_info = cpu_info,
        network_info = if network_info.is_empty() {
            "No network interfaces found".to_string()
        } else {
            network_info
        },
        disk_info = if disk_info.is_empty() {
            "No disk information available".to_string()
        } else {
            disk_info
        }
    );

    Ok(ctx.html(&html))
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
