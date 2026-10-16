use crate::config::load_config;
use serde_json::Value;

pub fn build_client() -> reqwest::Client {
    reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .expect("failed to build HTTP client")
}

pub fn auth() -> Result<(reqwest::Client, String, String), Box<dyn std::error::Error>> {
    let config = load_config().ok_or("not authenticated, run: csfx login")?;
    let token = config.token.ok_or("not authenticated, run: csfx login")?;
    Ok((build_client(), config.server, token))
}

pub fn base_url(server: &str) -> String {
    format!("{}/api", server.trim_end_matches('/'))
}

pub async fn get_json(
    client: &reqwest::Client,
    url: &str,
    token: &str,
) -> Result<Value, Box<dyn std::error::Error>> {
    let resp = client
        .get(url)
        .header("Authorization", format!("Bearer {}", token))
        .header("accept", "application/json")
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await?;
        crate::display::error(&format!("request failed: {} {}", status, body));
        std::process::exit(1);
    }

    Ok(resp.json().await?)
}

pub async fn delete_req(
    client: &reqwest::Client,
    url: &str,
    token: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let resp = client
        .delete(url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await?;
        crate::display::error(&format!("request failed: {} {}", status, body));
        std::process::exit(1);
    }

    Ok(())
}

pub async fn post_empty(
    client: &reqwest::Client,
    url: &str,
    token: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let resp = client
        .post(url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await?;
        crate::display::error(&format!("request failed: {} {}", status, body));
        std::process::exit(1);
    }

    Ok(())
}

#[allow(dead_code)]
pub async fn post_json(
    client: &reqwest::Client,
    url: &str,
    token: &str,
    body: &Value,
) -> Result<Value, Box<dyn std::error::Error>> {
    let resp = client
        .post(url)
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

    Ok(resp.json().await?)
}

