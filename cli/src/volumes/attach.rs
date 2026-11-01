use crate::display::{self, kv, kv_colored, section, status_color};
use crate::http::{auth, base_url, post_json};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
struct Volume {
    id: String,
    name: String,
    status: String,
    attached_to_agent: Option<String>,
    mapped_device: Option<String>,
}

pub async fn run(
    volume_id: &str,
    agent_id: &str,
    workload_id: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = auth()?;
    let url = format!("{}/volumes/{}/attach", base_url(&server), volume_id);

    let mut body = json!({ "agent_id": agent_id });
    if let Some(wid) = workload_id {
        body["workload_id"] = serde_json::Value::String(wid);
    }

    let pb = display::spinner("attaching volume...");
    let data = post_json(&client, &url, &token, &body).await;
    pb.finish_and_clear();

    let v: Volume = serde_json::from_value(data?)?;

    section("Volume Attached");
    kv("ID", &v.id);
    kv("Name", &v.name);
    kv_colored("Status", &v.status, status_color(&v.status));
    kv("Agent", v.attached_to_agent.as_deref().unwrap_or("-"));
    kv("Device", v.mapped_device.as_deref().unwrap_or("-"));
    println!();
    display::success("volume attached");

    Ok(())
}
