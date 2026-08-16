use crate::display::{self, kv, section, Table};
use crate::http::{auth, base_url, delete_req, get_json, post_json};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
struct AccessKey {
    id: String,
    name: String,
    garage_key_id: String,
    permissions: String,
    created_at: String,
}

#[derive(Deserialize)]
struct AccessKeyCreated {
    id: String,
    name: String,
    garage_key_id: String,
    permissions: String,
    secret_access_key: String,
}

pub async fn list(bucket_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = auth()?;
    let url = format!("{}/buckets/{}/keys", base_url(&server), bucket_id);

    let pb = display::spinner("fetching access keys...");
    let data = get_json(&client, &url, &token).await;
    pb.finish_and_clear();

    let keys: Vec<AccessKey> = serde_json::from_value(data?)?;

    display::section("Access Keys");

    if keys.is_empty() {
        display::info("no access keys found");
        return Ok(());
    }

    let mut table = Table::new(vec!["ID", "NAME", "ACCESS KEY ID", "PERMISSIONS", "CREATED"]);

    for k in &keys {
        table.add_row(vec![
            k.id[..8].to_string(),
            k.name.clone(),
            k.garage_key_id.clone(),
            k.permissions.clone(),
            k.created_at.clone(),
        ]);
    }

    table.print();
    println!();
    display::info(&format!("{} access key(s)", keys.len()));

    Ok(())
}

pub async fn create(
    bucket_id: &str,
    name: String,
    permissions: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = auth()?;
    let url = format!("{}/buckets/{}/keys", base_url(&server), bucket_id);

    let mut body = json!({ "name": name });
    if let Some(perm) = permissions {
        body["permissions"] = serde_json::Value::String(perm);
    }

    let pb = display::spinner("creating access key...");
    let data = post_json(&client, &url, &token, &body).await;
    pb.finish_and_clear();

    let k: AccessKeyCreated = serde_json::from_value(data?)?;
    print_created_key(&k.id, &k.name, &k.garage_key_id, &k.permissions, &k.secret_access_key);

    Ok(())
}

pub async fn rotate(bucket_id: &str, key_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = auth()?;
    let url = format!(
        "{}/buckets/{}/keys/{}/rotate",
        base_url(&server),
        bucket_id,
        key_id
    );

    let pb = display::spinner("rotating access key...");
    let data = post_json(&client, &url, &token, &json!({})).await;
    pb.finish_and_clear();

    let k: AccessKeyCreated = serde_json::from_value(data?)?;
    print_created_key(&k.id, &k.name, &k.garage_key_id, &k.permissions, &k.secret_access_key);

    Ok(())
}

pub async fn delete(bucket_id: &str, key_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = auth()?;
    let url = format!("{}/buckets/{}/keys/{}", base_url(&server), bucket_id, key_id);

    let pb = display::spinner("deleting access key...");
    let result = delete_req(&client, &url, &token).await;
    pb.finish_and_clear();

    result?;
    display::success(&format!("access key {} deleted", key_id));

    Ok(())
}

fn print_created_key(id: &str, name: &str, access_key_id: &str, permissions: &str, secret: &str) {
    section("Access Key Created");
    kv("ID", id);
    kv("Name", name);
    kv("Access Key ID", access_key_id);
    kv("Permissions", permissions);
    kv("Secret Access Key", secret);
    println!();
    display::warn("secret access key will not be shown again, store it now");
}
