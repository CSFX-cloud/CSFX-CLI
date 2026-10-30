use crate::display::{self, Table};
use crate::http::{auth, base_url, get_json};
use serde::Deserialize;

#[derive(Deserialize)]
struct Agent {
    id: String,
    hostname: String,
    ip_address: Option<String>,
    os_type: String,
    os_version: String,
    architecture: String,
    agent_version: String,
    status: String,
    last_heartbeat: Option<String>,
}

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = auth()?;
    let url = format!("{}/agents", base_url(&server));

    let pb = display::spinner("fetching nodes...");
    let data = get_json(&client, &url, &token).await;
    pb.finish_and_clear();

    let data = data?;
    let agents: Vec<Agent> = serde_json::from_value(data)?;

    display::section("Nodes");

    if agents.is_empty() {
        display::info("no nodes found");
        return Ok(());
    }

    let mut table = Table::new(vec![
        "ID", "HOSTNAME", "IP", "OS", "ARCH", "VERSION", "STATUS", "HEARTBEAT",
    ])
    .with_color(|col, val| {
        if col == 6 {
            display::status_color(val)
        } else {
            colored::Color::White
        }
    });

    for a in &agents {
        table.add_row(vec![
            a.id.clone(),
            a.hostname.clone(),
            a.ip_address.clone().unwrap_or_else(|| "-".to_string()),
            format!("{} {}", a.os_type, a.os_version),
            a.architecture.clone(),
            a.agent_version.clone(),
            a.status.clone(),
            a.last_heartbeat
                .clone()
                .map(|h| h[..16.min(h.len())].to_string())
                .unwrap_or_else(|| "never".to_string()),
        ]);
    }

    table.print();
    println!();
    display::info(&format!("{} node(s)", agents.len()));

    Ok(())
}
