use crate::display::{self, section, Table};
use crate::http::{auth, base_url, get_json, post_json};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
struct Policy {
    id: String,
    direction: String,
    action: String,
    source_cidr: Option<String>,
    destination_cidr: Option<String>,
    port: Option<i32>,
    protocol: Option<String>,
    priority: i32,
}

pub async fn list(network_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = auth()?;
    let url = format!("{}/networks/{}/policies", base_url(&server), network_id);

    let pb = display::spinner("fetching policies...");
    let data = get_json(&client, &url, &token).await;
    pb.finish_and_clear();

    let data = data?;
    let policies: Vec<Policy> = serde_json::from_value(data)?;

    section("Networks  /  Policies");

    if policies.is_empty() {
        display::info("no policies");
        return Ok(());
    }

    let mut table = Table::new(vec![
        "ID", "DIR", "ACTION", "SRC", "DST", "PORT", "PROTO", "PRIO",
    ])
    .with_color(|col, val| {
        if col == 2 {
            if val == "allow" {
                colored::Color::Green
            } else {
                colored::Color::Red
            }
        } else {
            colored::Color::White
        }
    });

    for p in &policies {
        table.add_row(vec![
            p.id[..8].to_string(),
            p.direction.clone(),
            p.action.clone(),
            p.source_cidr.clone().unwrap_or_else(|| "*".to_string()),
            p.destination_cidr
                .clone()
                .unwrap_or_else(|| "*".to_string()),
            p.port
                .map(|v| v.to_string())
                .unwrap_or_else(|| "*".to_string()),
            p.protocol.clone().unwrap_or_else(|| "*".to_string()),
            p.priority.to_string(),
        ]);
    }

    table.print();
    println!();
    display::info(&format!("{} policy(s)", policies.len()));

    Ok(())
}

pub async fn create(
    network_id: &str,
    direction: String,
    action: String,
    priority: i32,
    source_cidr: Option<String>,
    destination_cidr: Option<String>,
    port: Option<i32>,
    protocol: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = auth()?;
    let url = format!("{}/networks/{}/policies", base_url(&server), network_id);

    let body = json!({
        "direction": direction,
        "action": action,
        "priority": priority,
        "source_cidr": source_cidr,
        "destination_cidr": destination_cidr,
        "port": port,
        "protocol": protocol,
    });

    let pb = display::spinner("creating policy...");
    let data = post_json(&client, &url, &token, &body).await;
    pb.finish_and_clear();

    data?;

    section("Networks  /  Policy Create");
    display::success("policy created");

    Ok(())
}
