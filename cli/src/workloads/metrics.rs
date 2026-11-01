use crate::display::{self, kv, kv_colored, section, status_color};
use crate::http::{auth, base_url, get_json};
use colored::Colorize;
use serde::Deserialize;

#[derive(Deserialize)]
struct Workload {
    id: String,
    name: String,
    image: String,
    cpu_millicores: i32,
    memory_bytes: i64,
    disk_bytes: i64,
    status: String,
    assigned_agent_id: Option<String>,
}

#[derive(Deserialize)]
struct AgentMetrics {
    cpu_usage_percent: Option<f32>,
    memory_total_bytes: Option<i64>,
    memory_used_bytes: Option<i64>,
    disk_total_bytes: Option<i64>,
    disk_used_bytes: Option<i64>,
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

fn print_workload_metrics(w: &Workload, agent: Option<&AgentMetrics>) {
    section(&format!("Workload Metrics  /  {}", w.name));

    kv("ID", &w.id);
    kv("Image", &w.image);
    kv_colored("Status", &w.status, status_color(&w.status));

    println!();

    kv("Allocated CPU", &format!("{} millicores", w.cpu_millicores));
    kv(
        "Allocated Memory",
        &format!("{} MB", w.memory_bytes / 1024 / 1024),
    );
    kv(
        "Allocated Disk",
        &format!("{} GB", w.disk_bytes / 1024 / 1024 / 1024),
    );

    if let Some(m) = agent {
        println!();
        println!("  {}", "Node Metrics (host agent)".dimmed());

        if let Some(pct) = m.cpu_usage_percent {
            println!("  {:<20} {}", "Node CPU".dimmed(), usage_bar(pct, 30));
        }
        if let (Some(total), Some(used)) = (m.memory_total_bytes, m.memory_used_bytes) {
            let pct = if total > 0 {
                used as f32 / total as f32 * 100.0
            } else {
                0.0
            };
            println!("  {:<20} {}", "Node Memory".dimmed(), usage_bar(pct, 30));
            kv(
                "  Node Mem Used",
                &format!(
                    "{:.1}G / {:.1}G",
                    used as f64 / 1_073_741_824.0,
                    total as f64 / 1_073_741_824.0
                ),
            );
        }
        if let (Some(total), Some(used)) = (m.disk_total_bytes, m.disk_used_bytes) {
            let pct = if total > 0 {
                used as f32 / total as f32 * 100.0
            } else {
                0.0
            };
            println!("  {:<20} {}", "Node Disk".dimmed(), usage_bar(pct, 30));
        }
    }

    println!();
}

pub async fn run(id: &str, watch: bool) -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = auth()?;
    let workload_url = format!("{}/workloads/{}", base_url(&server), id);

    if !watch {
        let pb = display::spinner("fetching workload metrics...");
        let data = get_json(&client, &workload_url, &token).await;
        pb.finish_and_clear();

        let w: Workload = serde_json::from_value(data?)?;

        let agent_metrics = if let Some(ref agent_id) = w.assigned_agent_id {
            let url = format!("{}/agents/{}/metrics/latest", base_url(&server), agent_id);
            get_json(&client, &url, &token)
                .await
                .ok()
                .and_then(|v| serde_json::from_value::<AgentMetrics>(v).ok())
        } else {
            None
        };

        print_workload_metrics(&w, agent_metrics.as_ref());
        return Ok(());
    }

    loop {
        let data = get_json(&client, &workload_url, &token).await;
        print!("\x1B[2J\x1B[H");

        match data {
            Ok(v) => match serde_json::from_value::<Workload>(v) {
                Ok(w) => {
                    let agent_metrics = if let Some(ref agent_id) = w.assigned_agent_id {
                        let url =
                            format!("{}/agents/{}/metrics/latest", base_url(&server), agent_id);
                        get_json(&client, &url, &token)
                            .await
                            .ok()
                            .and_then(|v| serde_json::from_value::<AgentMetrics>(v).ok())
                    } else {
                        None
                    };
                    print_workload_metrics(&w, agent_metrics.as_ref());
                }
                Err(e) => display::error(&format!("parse error: {}", e)),
            },
            Err(e) => display::error(&format!("fetch error: {}", e)),
        }

        println!("  {}", "refreshing every 5s — Ctrl+C to exit".dimmed());
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}
