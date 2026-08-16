use crate::display::{self, kv, kv_colored, section, status_color};
use crate::http::{auth, base_url, get_json};
use serde::Deserialize;

#[derive(Deserialize)]
struct Bucket {
    id: String,
    name: String,
    global_alias: String,
    exposure: String,
    quota_max_size: Option<i64>,
    quota_max_objects: Option<i64>,
    status: String,
    created_at: String,
    updated_at: Option<String>,
}

pub async fn run(id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = auth()?;
    let url = format!("{}/buckets/{}", base_url(&server), id);

    let pb = display::spinner("fetching bucket...");
    let data = get_json(&client, &url, &token).await;
    pb.finish_and_clear();

    let b: Bucket = serde_json::from_value(data?)?;

    section("Bucket");

    kv("ID", &b.id);
    kv("Name", &b.name);
    kv("Global Alias", &b.global_alias);
    kv("Exposure", &b.exposure);
    kv(
        "Quota Size",
        &b.quota_max_size
            .map(|s| format!("{}B", s))
            .unwrap_or_else(|| "-".to_string()),
    );
    kv(
        "Quota Objects",
        &b.quota_max_objects
            .map(|s| s.to_string())
            .unwrap_or_else(|| "-".to_string()),
    );
    kv_colored("Status", &b.status, status_color(&b.status));
    kv("Created", &b.created_at);
    kv("Updated", b.updated_at.as_deref().unwrap_or("-"));

    println!();
    Ok(())
}
