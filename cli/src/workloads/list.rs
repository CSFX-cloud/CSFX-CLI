use crate::display::{self, section, status_color, Table};
use crate::http::{auth, base_url, get_json};
use serde::Deserialize;

#[derive(Deserialize)]
struct Workload {
    id: String,
    name: String,
    image: String,
    cpu_millicores: i32,
    memory_bytes: i64,
    status: String,
    assigned_agent_id: Option<String>,
    container_id: Option<String>,
}

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = auth()?;
    let url = format!("{}/workloads", base_url(&server));

    let pb = display::spinner("fetching workloads...");
    let data = get_json(&client, &url, &token).await;
    pb.finish_and_clear();

    let data = data?;
    let workloads: Vec<Workload> = serde_json::from_value(data)?;

    section("Workloads");

    if workloads.is_empty() {
        display::info("no workloads");
        return Ok(());
    }

    let mut table = Table::new(vec![
        "ID",
        "NAME",
        "IMAGE",
        "CPU(m)",
        "MEM(MB)",
        "STATUS",
        "AGENT",
        "CONTAINER",
    ])
    .with_color(|col, val| {
        if col == 5 {
            status_color(val)
        } else {
            colored::Color::White
        }
    });

    for w in &workloads {
        table.add_row(vec![
            w.id[..8].to_string(),
            w.name.clone(),
            w.image.clone(),
            w.cpu_millicores.to_string(),
            (w.memory_bytes / 1024 / 1024).to_string(),
            w.status.clone(),
            w.assigned_agent_id
                .as_deref()
                .map(|id| &id[..8])
                .unwrap_or("-")
                .to_string(),
            w.container_id
                .as_deref()
                .map(|id| &id[..12.min(id.len())])
                .unwrap_or("-")
                .to_string(),
        ]);
    }

    table.print();
    println!();
    display::info(&format!("{} workload(s)", workloads.len()));

    Ok(())
}
