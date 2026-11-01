use crate::{display, http};
use serde_json::json;

pub async fn run(
    username: String,
    password: String,
    role_id: String,
    email: Option<String>,
    force_password_change: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = http::auth()?;
    let url = format!("{}/organization/users", http::base_url(&server));

    let body = json!({
        "username": username,
        "password": password,
        "role_id": role_id,
        "email": email,
        "force_password_change": force_password_change,
    });

    let pb = display::spinner("creating user...");
    let u = http::post_json(&client, &url, &token, &body).await?;
    pb.finish_and_clear();

    display::success(&format!(
        "user created: {}",
        u["id"].as_str().unwrap_or("-")
    ));
    display::kv("Username", u["username"].as_str().unwrap_or("-"));
    display::kv("Role", u["role_name"].as_str().unwrap_or("-"));
    println!();
    Ok(())
}
