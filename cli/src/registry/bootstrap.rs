use crate::display::{self, kv, Table};
use crate::http::{auth, base_url, get_json, post_json};
use colored::Colorize;
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
struct BootstrapToken {
    id: String,
    token: String,
    description: Option<String>,
    created_by: String,
    expires_at: String,
    max_uses: i32,
    use_count: i32,
    revoked: bool,
}

pub async fn create(
    description: Option<String>,
    ttl_hours: Option<i64>,
    max_uses: Option<i32>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = auth()?;
    let url = format!("{}/registry/admin/bootstrap-tokens", base_url(&server));

    let body = json!({
        "description": description,
        "ttl_hours": ttl_hours.unwrap_or(720),
        "max_uses": max_uses.unwrap_or(1000),
    });

    let pb = display::spinner("creating bootstrap token...");
    let data = post_json(&client, &url, &token, &body).await;
    pb.finish_and_clear();

    let data = data?;
    let bt: BootstrapToken = serde_json::from_value(data)?;

    display::section("Registry  /  Bootstrap Token Created");

    kv("ID", &bt.id);
    kv("Max Uses", &bt.max_uses.to_string());
    kv("Expires", &bt.expires_at);
    if let Some(desc) = &bt.description {
        kv("Description", desc);
    }
    println!();
    println!("  {}", "Bootstrap Token".bold());
    println!("  {}", bt.token.cyan().bold());
    println!();
    display::info("embed this token in services.csf-daemon.registrationToken in your NixOS config");
    display::info("the token is valid for all nodes until TTL expires or max_uses is reached");

    Ok(())
}

pub async fn list() -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = auth()?;
    let url = format!("{}/registry/admin/bootstrap-tokens", base_url(&server));

    let pb = display::spinner("fetching bootstrap tokens...");
    let data = get_json(&client, &url, &token).await;
    pb.finish_and_clear();

    let data = data?;
    let tokens: Vec<BootstrapToken> = serde_json::from_value(data)?;

    display::section("Registry  /  Bootstrap Tokens");

    if tokens.is_empty() {
        display::info("no active bootstrap tokens");
        return Ok(());
    }

    let mut table = Table::new(vec![
        "ID",
        "DESCRIPTION",
        "CREATED BY",
        "USES",
        "EXPIRES",
        "REVOKED",
    ])
    .with_color(|col, val| {
        if col == 5 {
            if val == "yes" {
                colored::Color::Red
            } else {
                colored::Color::Green
            }
        } else {
            colored::Color::White
        }
    });

    for bt in &tokens {
        table.add_row(vec![
            bt.id[..8].to_string(),
            bt.description.clone().unwrap_or_else(|| "-".to_string()),
            bt.created_by.clone(),
            format!("{}/{}", bt.use_count, bt.max_uses),
            bt.expires_at.clone(),
            if bt.revoked { "yes" } else { "no" }.to_string(),
        ]);
    }

    table.print();
    println!();
    display::info(&format!("{} bootstrap token(s)", tokens.len()));

    Ok(())
}

pub async fn revoke(id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = auth()?;
    let url = format!(
        "{}/registry/admin/bootstrap-tokens/{}/revoke",
        base_url(&server),
        id
    );

    let pb = display::spinner("revoking bootstrap token...");
    let data = post_json(&client, &url, &token, &json!({})).await;
    pb.finish_and_clear();

    data?;

    display::section("Registry  /  Bootstrap Token Revoked");
    kv("ID", id);
    display::info("no new nodes can register using this token");

    Ok(())
}
