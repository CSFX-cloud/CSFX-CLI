use crate::display::{self, kv, kv_colored, section, status_color};
use crate::http::{auth, base_url, get_json};
use serde::Deserialize;

#[derive(Deserialize)]
struct Volume {
    id: String,
    name: String,
    size_gb: i32,
    pool: String,
    image_name: String,
    status: String,
    attached_to_agent: Option<String>,
    attached_to_workload: Option<String>,
    mapped_device: Option<String>,
    created_at: String,
    updated_at: Option<String>,
}

pub async fn run(id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = auth()?;
    let url = format!("{}/volumes/{}", base_url(&server), id);

    let pb = display::spinner("fetching volume...");
    let data = get_json(&client, &url, &token).await;
    pb.finish_and_clear();

    let v: Volume = serde_json::from_value(data?)?;

    section("Volume");

    kv("ID", &v.id);
    kv("Name", &v.name);
    kv("Size", &format!("{}G", v.size_gb));
    kv("Pool", &v.pool);
    kv("Image", &v.image_name);
    kv_colored("Status", &v.status, status_color(&v.status));
    kv("Agent", v.attached_to_agent.as_deref().unwrap_or("-"));
    kv("Workload", v.attached_to_workload.as_deref().unwrap_or("-"));
    kv("Device", v.mapped_device.as_deref().unwrap_or("-"));
    kv("Created", &v.created_at);
    kv("Updated", v.updated_at.as_deref().unwrap_or("-"));

    println!();
    Ok(())
}
