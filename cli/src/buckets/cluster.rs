use crate::display::{self, kv, kv_colored, section, Table};
use crate::http::{auth, base_url, get_json};
use colored::Color;
use serde::Deserialize;

#[derive(Deserialize)]
struct GarageNode {
    agent_id: String,
    zone: String,
    role: String,
    status: String,
    capacity_bytes: Option<i64>,
}

#[derive(Deserialize)]
struct ClusterStatus {
    storage_node_count: u32,
    replication_factor: u32,
    degraded: bool,
    nodes: Vec<GarageNode>,
}

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = auth()?;
    let url = format!("{}/object-storage/cluster", base_url(&server));

    let pb = display::spinner("fetching cluster status...");
    let data = get_json(&client, &url, &token).await;
    pb.finish_and_clear();

    let status: ClusterStatus = serde_json::from_value(data?)?;

    section("Object Storage Cluster");
    kv("Storage Nodes", &status.storage_node_count.to_string());
    kv("Replication Factor", &status.replication_factor.to_string());
    kv_colored(
        "Degraded",
        if status.degraded { "yes" } else { "no" },
        if status.degraded { Color::Red } else { Color::Green },
    );
    println!();

    if status.nodes.is_empty() {
        display::info("no garage nodes registered");
        return Ok(());
    }

    let mut table = Table::new(vec!["AGENT", "ZONE", "ROLE", "STATUS", "CAPACITY"]).with_color(
        |col, val| {
            if col == 3 {
                display::status_color(val)
            } else {
                Color::White
            }
        },
    );

    for n in &status.nodes {
        table.add_row(vec![
            n.agent_id[..8].to_string(),
            n.zone.clone(),
            n.role.clone(),
            n.status.clone(),
            n.capacity_bytes
                .map(|c| format!("{}B", c))
                .unwrap_or_else(|| "gateway".to_string()),
        ]);
    }

    table.print();

    Ok(())
}
