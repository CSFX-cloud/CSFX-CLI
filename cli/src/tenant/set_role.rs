use crate::{display, http};
use reqwest::Client;
use serde_json::json;

pub async fn run(user_id: &str, role_id: String) -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = http::auth()?;
    let url = format!(
        "{}/organization/users/{}/role",
        http::base_url(&server),
        user_id
    );

    let body = json!({ "role_id": role_id });

    let pb = display::spinner("updating role...");
    put_json(&client, &url, &token, &body).await?;
    pb.finish_and_clear();

    display::success(&format!("role updated for user {}", user_id));
    println!();
    Ok(())
}

async fn put_json(
    client: &Client,
    url: &str,
    token: &str,
    body: &serde_json::Value,
) -> Result<(), Box<dyn std::error::Error>> {
    let resp = client
        .put(url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(body)
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let err = resp.text().await?;
        crate::display::error(&format!("request failed: {} {}", status, err));
        std::process::exit(1);
    }

    Ok(())
}
