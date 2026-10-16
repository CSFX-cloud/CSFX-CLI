use crate::display::{self, kv};
use crate::http::{auth, base_url, delete_req, post_empty, post_json};
use colored::Colorize;
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
struct JoinToken {
    join_string: String,
    role: String,
    expires_at: String,
}

pub async fn join_token(ttl_hours: i64) -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = auth()?;
    let url = format!("{}/cluster/join-tokens", base_url(&server));
    let body = json!({ "ttl_hours": ttl_hours });

    let pb = display::spinner("creating join token...");
    let data = post_json(&client, &url, &token, &body).await;
    pb.finish_and_clear();

    let join: JoinToken = serde_json::from_value(data?)?;

    display::section("Nodes  /  Join Token");
    kv("Role", &join.role);
    kv("Expires", &join.expires_at);
    println!();
    println!("  {}", join.join_string.cyan().bold());
    println!();
    display::info("set CSFX_JOIN to this value on the new node, valid for a single use");

    Ok(())
}

pub async fn cordon(id: &str) -> Result<(), Box<dyn std::error::Error>> {
    node_action(id, "cordon", "cordoning node...", "cordoned").await
}

pub async fn uncordon(id: &str) -> Result<(), Box<dyn std::error::Error>> {
    node_action(id, "uncordon", "uncordoning node...", "uncordoned").await
}

pub async fn drain(id: &str) -> Result<(), Box<dyn std::error::Error>> {
    node_action(id, "drain", "draining node...", "draining").await
}

pub async fn remove(id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = auth()?;
    let url = format!("{}/cluster/nodes/{}", base_url(&server), id);

    let pb = display::spinner("removing node...");
    let result = delete_req(&client, &url, &token).await;
    pb.finish_and_clear();

    result?;
    display::success(&format!("node {} removed", id));

    Ok(())
}

async fn node_action(
    id: &str,
    action: &str,
    progress: &str,
    done: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = auth()?;
    let url = format!("{}/agents/{}/{}", base_url(&server), id, action);

    let pb = display::spinner(progress);
    let result = post_empty(&client, &url, &token).await;
    pb.finish_and_clear();

    result?;
    display::success(&format!("node {} {}", id, done));

    Ok(())
}
