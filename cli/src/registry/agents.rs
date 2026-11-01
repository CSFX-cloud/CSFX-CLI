use crate::display::{self, kv, kv_colored, section, status_color, Table};
use crate::http::{auth, base_url, get_json};
use serde::Deserialize;

#[derive(Deserialize)]
struct Agent {
    id: String,
    name: String,
    hostname: String,
    ip_address: Option<String>,
    agent_version: String,
    os_type: String,
    architecture: String,
    status: String,
    last_heartbeat: Option<String>,
    registered_at: String,
}

pub async fn list() -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = auth()?;
    let url = format!("{}/registry/admin/agents", base_url(&server));

    let pb = display::spinner("fetching agents...");
    let data = get_json(&client, &url, &token).await;
    pb.finish_and_clear();

    let data = data?;
    let agents: Vec<Agent> = serde_json::from_value(data)?;

    section("Registry  /  Agents");

    if agents.is_empty() {
        display::info("no agents registered");
        return Ok(());
    }

    let mut table = Table::new(vec![
        "ID", "HOSTNAME", "IP", "OS", "ARCH", "VERSION", "STATUS",
    ])
    .with_color(|col, val| {
        if col == 6 {
            status_color(val)
        } else {
            colored::Color::White
        }
    });

    for a in &agents {
        table.add_row(vec![
            a.id[..8].to_string(),
            a.hostname.clone(),
            a.ip_address.clone().unwrap_or_else(|| "-".to_string()),
            a.os_type.clone(),
            a.architecture.clone(),
            a.agent_version.clone(),
            a.status.clone(),
        ]);
    }

    table.print();
    println!();
    display::info(&format!("{} agent(s)", agents.len()));

    Ok(())
}

pub async fn get(id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = auth()?;
    let url = format!("{}/registry/admin/agents/{}", base_url(&server), id);

    let pb = display::spinner("fetching agent...");
    let data = get_json(&client, &url, &token).await;
    pb.finish_and_clear();

    let data = data?;
    let a: Agent = serde_json::from_value(data)?;

    section("Registry  /  Agent");

    kv("ID", &a.id);
    kv("Name", &a.name);
    kv("Hostname", &a.hostname);
    kv("IP", &a.ip_address.unwrap_or_else(|| "-".to_string()));
    kv("OS", &a.os_type);
    kv("Architecture", &a.architecture);
    kv("Agent Version", &a.agent_version);
    kv_colored("Status", &a.status, status_color(&a.status));
    kv(
        "Last Heartbeat",
        &a.last_heartbeat.unwrap_or_else(|| "never".to_string()),
    );
    kv("Registered", &a.registered_at);

    println!();
    Ok(())
}
