use crate::display::{self, kv, kv_colored, section, status_color};
use crate::http::{auth, base_url, get_json};
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
    container_id: Option<String>,
    created_at: String,
    updated_at: Option<String>,
}

pub async fn run(id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = auth()?;
    let url = format!("{}/workloads/{}", base_url(&server), id);

    let pb = display::spinner("fetching workload...");
    let data = get_json(&client, &url, &token).await;
    pb.finish_and_clear();

    let data = data?;
    let w: Workload = serde_json::from_value(data)?;

    section("Workloads  /  Get");

    kv("ID", &w.id);
    kv("Name", &w.name);
    kv("Image", &w.image);
    kv_colored("Status", &w.status, status_color(&w.status));
    kv("CPU", &format!("{} millicores", w.cpu_millicores));
    kv("Memory", &format!("{} MB", w.memory_bytes / 1024 / 1024));
    kv("Disk", &format!("{} GB", w.disk_bytes / 1024 / 1024 / 1024));
    kv("Agent", w.assigned_agent_id.as_deref().unwrap_or("-"));
    kv("Container", w.container_id.as_deref().unwrap_or("-"));
    kv("Created", &w.created_at[..16.min(w.created_at.len())]);
    kv(
        "Updated",
        w.updated_at
            .as_deref()
            .map(|t| &t[..16.min(t.len())])
            .unwrap_or("-"),
    );

    println!();

    Ok(())
}
