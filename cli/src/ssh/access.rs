use chrono::{DateTime, Local, Utc};
use serde::Deserialize;
use serde_json::json;

use crate::display;
use crate::http::{auth, base_url, get_json};

pub const DEFAULT_DURATION_HOURS: i64 = 8;

#[derive(Deserialize)]
pub struct SshAccess {
    pub expires_at: Option<DateTime<Utc>>,
    pub effective: bool,
}

pub async fn fetch(
    client: &reqwest::Client,
    base: &str,
    token: &str,
) -> Result<SshAccess, Box<dyn std::error::Error>> {
    let value = get_json(client, &format!("{}/admin/settings/ssh", base), token).await?;
    Ok(serde_json::from_value(value)?)
}

fn print(access: &SshAccess) {
    let state = if access.effective {
        "enabled"
    } else {
        "disabled"
    };
    display::kv("SSH access", state);
    match (access.effective, access.expires_at) {
        (true, Some(expires_at)) => display::kv(
            "Expires",
            &expires_at
                .with_timezone(&Local)
                .format("%Y-%m-%d %H:%M")
                .to_string(),
        ),
        (true, None) => display::kv("Expires", "never"),
        _ => {}
    }
}

pub async fn status() -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = auth()?;
    print(&fetch(&client, &base_url(&server), &token).await?);
    Ok(())
}

async fn update(
    enabled: bool,
    duration_hours: Option<i64>,
) -> Result<SshAccess, Box<dyn std::error::Error>> {
    let (client, server, token) = auth()?;
    let response = client
        .put(format!("{}/admin/settings/ssh", base_url(&server)))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({ "enabled": enabled, "duration_hours": duration_hours }))
        .send()
        .await?;
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await?;
        return Err(format!("request failed: {} {}", status, body).into());
    }
    Ok(response.json().await?)
}

pub async fn enable(hours: Option<i64>, unlimited: bool) -> Result<(), Box<dyn std::error::Error>> {
    let duration = if unlimited {
        None
    } else {
        Some(hours.unwrap_or(DEFAULT_DURATION_HOURS))
    };
    print(&update(true, duration).await?);
    display::info("connect with: csfx ssh connect <node>");
    Ok(())
}

pub async fn disable() -> Result<(), Box<dyn std::error::Error>> {
    print(&update(false, None).await?);
    Ok(())
}
