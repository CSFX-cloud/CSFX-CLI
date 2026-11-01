use crate::display::{self, kv, kv_colored, section, status_color};
use crate::http::{auth, base_url, post_json};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
struct Volume {
    id: String,
    name: String,
    size_gb: i32,
    pool: String,
    status: String,
}

pub async fn run(
    name: String,
    size_gb: i32,
    pool: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = auth()?;
    let url = format!("{}/volumes", base_url(&server));

    let mut body = json!({ "name": name, "size_gb": size_gb });
    if let Some(p) = pool {
        body["pool"] = serde_json::Value::String(p);
    }

    let pb = display::spinner("creating volume...");
    let data = post_json(&client, &url, &token, &body).await;
    pb.finish_and_clear();

    let v: Volume = serde_json::from_value(data?)?;

    section("Volume Created");
    kv("ID", &v.id);
    kv("Name", &v.name);
    kv("Size", &format!("{}G", v.size_gb));
    kv("Pool", &v.pool);
    kv_colored("Status", &v.status, status_color(&v.status));
    println!();
    display::success("volume created");

    Ok(())
}
