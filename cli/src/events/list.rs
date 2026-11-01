use crate::display::{self, section, status_color, Table};
use crate::http::{auth, base_url, get_json};
use serde::Deserialize;

#[derive(Deserialize)]
struct FailoverEvent {
    id: String,
    agent_id: Option<String>,
    event_type: String,
    affected_workloads: Option<Vec<String>>,
    duration_ms: Option<i64>,
    created_at: String,
}

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = auth()?;
    let url = format!("{}/events", base_url(&server));

    let pb = display::spinner("fetching events...");
    let data = get_json(&client, &url, &token).await;
    pb.finish_and_clear();

    let data = data?;
    let events: Vec<FailoverEvent> = serde_json::from_value(data)?;

    section("Events");

    if events.is_empty() {
        display::info("no events");
        return Ok(());
    }

    let mut table = Table::new(vec![
        "ID",
        "TYPE",
        "AGENT",
        "WORKLOADS",
        "DURATION",
        "CREATED",
    ])
    .with_color(|col, val| {
        if col == 1 {
            status_color(val)
        } else {
            colored::Color::White
        }
    });

    for e in &events {
        let duration = e
            .duration_ms
            .map(|ms| format!("{}ms", ms))
            .unwrap_or_else(|| "-".to_string());

        let workload_count = e
            .affected_workloads
            .as_ref()
            .map(|w| w.len().to_string())
            .unwrap_or_else(|| "-".to_string());

        table.add_row(vec![
            e.id[..8].to_string(),
            e.event_type.clone(),
            e.agent_id
                .as_deref()
                .map(|id| &id[..8])
                .unwrap_or("-")
                .to_string(),
            workload_count,
            duration,
            e.created_at[..16.min(e.created_at.len())].to_string(),
        ]);
    }

    table.print();
    println!();
    display::info(&format!("{} event(s)", events.len()));

    Ok(())
}
