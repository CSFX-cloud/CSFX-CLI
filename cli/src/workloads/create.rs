use crate::display::{self, kv, kv_colored, section, status_color};
use crate::http::{auth, base_url, post_json};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
struct CreateResponse {
    workload_id: String,
    status: String,
    assigned_agent_id: Option<String>,
    message: String,
}

pub async fn run(
    name: String,
    image: String,
    cpu: i32,
    memory: i64,
    disk: i64,
) -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = auth()?;
    let url = format!("{}/workloads", base_url(&server));

    let body = json!({
        "name": name,
        "image": image,
        "cpu_millicores": cpu,
        "memory_bytes": memory,
        "disk_bytes": disk,
    });

    let pb = display::spinner("scheduling workload...");
    let data = post_json(&client, &url, &token, &body).await;
    pb.finish_and_clear();

    let data = data?;
    let resp: CreateResponse = serde_json::from_value(data)?;

    section("Workloads  /  Create");

    kv("Workload ID", &resp.workload_id);
    kv_colored("Status", &resp.status, status_color(&resp.status));
    kv(
        "Assigned Agent",
        resp.assigned_agent_id
            .as_deref()
            .unwrap_or("none (pending)"),
    );
    kv("Message", &resp.message);

    println!();

    if resp.assigned_agent_id.is_some() {
        display::success("workload scheduled");
    } else {
        display::warn("no suitable agent available, workload is pending");
    }

    Ok(())
}
