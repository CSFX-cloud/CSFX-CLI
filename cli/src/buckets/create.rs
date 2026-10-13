use crate::display::{self, kv, kv_colored, section, status_color};
use crate::http::{auth, base_url, post_json};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
struct Bucket {
    id: String,
    name: String,
    global_alias: String,
    exposure: String,
    status: String,
}

#[allow(clippy::too_many_arguments)]
pub async fn run(
    name: String,
    resource_group: Option<String>,
    exposure: Option<String>,
    quota_size: Option<i64>,
    quota_objects: Option<i64>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = auth()?;
    let url = format!("{}/buckets", base_url(&server));

    let mut body = json!({ "name": name });
    if let Some(rg) = resource_group {
        body["resource_group_id"] = serde_json::Value::String(rg);
    }
    if let Some(exp) = exposure {
        body["exposure"] = serde_json::Value::String(exp);
    }
    if let Some(size) = quota_size {
        body["quota_max_size"] = serde_json::Value::Number(size.into());
    }
    if let Some(objects) = quota_objects {
        body["quota_max_objects"] = serde_json::Value::Number(objects.into());
    }

    let pb = display::spinner("creating bucket...");
    let data = post_json(&client, &url, &token, &body).await;
    pb.finish_and_clear();

    let b: Bucket = serde_json::from_value(data?)?;

    section("Bucket Created");
    kv("ID", &b.id);
    kv("Name", &b.name);
    kv("Global Alias", &b.global_alias);
    kv("Exposure", &b.exposure);
    kv_colored("Status", &b.status, status_color(&b.status));
    println!();
    display::success("bucket created");

    Ok(())
}
