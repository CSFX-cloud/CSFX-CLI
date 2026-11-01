use crate::display::{self, section, status_color, Table};
use crate::http::{auth, base_url, get_json};
use colored::Colorize;
use serde::Deserialize;

#[derive(Deserialize)]
struct NodeStats {
    agent_id: String,
    hostname: String,
    status: String,
    cpu_usage_percent: Option<f32>,
    memory_total_bytes: Option<i64>,
    memory_used_bytes: Option<i64>,
    disk_total_bytes: Option<i64>,
    disk_used_bytes: Option<i64>,
}

#[derive(Deserialize)]
struct ClusterStats {
    node_count: usize,
    online_count: usize,
    total_cpu_cores: i64,
    avg_cpu_usage_percent: f32,
    total_memory_bytes: i64,
    used_memory_bytes: i64,
    total_disk_bytes: i64,
    used_disk_bytes: i64,
    nodes: Vec<NodeStats>,
}

fn bytes_to_gb(bytes: i64) -> f64 {
    bytes as f64 / 1_073_741_824.0
}

fn usage_bar(pct: f32, width: usize) -> String {
    let filled = ((pct / 100.0) * width as f32) as usize;
    let empty = width - filled.min(width);
    let color = if pct > 90.0 {
        colored::Color::Red
    } else if pct > 70.0 {
        colored::Color::Yellow
    } else {
        colored::Color::Green
    };
    format!(
        "[{}{}] {:.1}%",
        "#".repeat(filled).color(color),
        ".".repeat(empty).dimmed(),
        pct
    )
}

fn pct_cell(pct: f32) -> String {
    let color = if pct > 90.0 {
        colored::Color::Red
    } else if pct > 70.0 {
        colored::Color::Yellow
    } else {
        colored::Color::Green
    };
    format!("{:.1}%", pct).color(color).to_string()
}

fn print_stats(stats: &ClusterStats) {
    section("Cluster Stats");

    println!(
        "  {:<20} {} / {} online",
        "Nodes".dimmed(),
        stats.online_count,
        stats.node_count
    );
    println!(
        "  {:<20} {} cores",
        "Total CPU".dimmed(),
        stats.total_cpu_cores
    );
    println!(
        "  {:<20} {}",
        "Avg CPU".dimmed(),
        usage_bar(stats.avg_cpu_usage_percent, 30)
    );
    println!(
        "  {:<20} {:.1}G / {:.1}G",
        "Total Memory".dimmed(),
        bytes_to_gb(stats.used_memory_bytes),
        bytes_to_gb(stats.total_memory_bytes)
    );
    println!(
        "  {:<20} {:.1}G / {:.1}G",
        "Total Disk".dimmed(),
        bytes_to_gb(stats.used_disk_bytes),
        bytes_to_gb(stats.total_disk_bytes)
    );

    section("Node Metrics");

    if stats.nodes.is_empty() {
        display::info("no nodes");
        return;
    }

    let mut table = Table::new(vec![
        "ID",
        "HOSTNAME",
        "STATUS",
        "CPU",
        "MEM USED",
        "MEM TOTAL",
        "DISK USED",
        "DISK TOTAL",
    ])
    .with_color(|col, val| {
        if col == 2 {
            status_color(val)
        } else {
            colored::Color::White
        }
    });

    for n in &stats.nodes {
        let cpu = n
            .cpu_usage_percent
            .map(|p| pct_cell(p))
            .unwrap_or_else(|| "-".to_string());
        let mem_used = n
            .memory_used_bytes
            .map(|b| format!("{:.1}G", bytes_to_gb(b)))
            .unwrap_or_else(|| "-".to_string());
        let mem_total = n
            .memory_total_bytes
            .map(|b| format!("{:.1}G", bytes_to_gb(b)))
            .unwrap_or_else(|| "-".to_string());
        let disk_used = n
            .disk_used_bytes
            .map(|b| format!("{:.1}G", bytes_to_gb(b)))
            .unwrap_or_else(|| "-".to_string());
        let disk_total = n
            .disk_total_bytes
            .map(|b| format!("{:.1}G", bytes_to_gb(b)))
            .unwrap_or_else(|| "-".to_string());

        table.add_row(vec![
            n.agent_id[..8].to_string(),
            n.hostname.clone(),
            n.status.clone(),
            cpu,
            mem_used,
            mem_total,
            disk_used,
            disk_total,
        ]);
    }

    table.print();
    println!();
}

pub async fn run(watch: bool) -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = auth()?;
    let url = format!("{}/system/stats", base_url(&server));

    if !watch {
        let pb = display::spinner("fetching cluster metrics...");
        let data = get_json(&client, &url, &token).await;
        pb.finish_and_clear();
        let stats: ClusterStats = serde_json::from_value(data?)?;
        print_stats(&stats);
        return Ok(());
    }

    loop {
        let data = get_json(&client, &url, &token).await;
        print!("\x1B[2J\x1B[H");
        match data {
            Ok(v) => match serde_json::from_value::<ClusterStats>(v) {
                Ok(stats) => print_stats(&stats),
                Err(e) => display::error(&format!("parse error: {}", e)),
            },
            Err(e) => display::error(&format!("fetch error: {}", e)),
        }
        println!("  {}", "refreshing every 5s — Ctrl+C to exit".dimmed());
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}
