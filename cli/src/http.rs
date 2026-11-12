use crate::config::load_config;
use serde_json::Value;
use std::sync::atomic::{AtomicBool, Ordering};

static INSECURE: AtomicBool = AtomicBool::new(false);

pub fn set_insecure(insecure: bool) {
    INSECURE.store(insecure, Ordering::Relaxed);
}

pub fn is_insecure() -> bool {
    INSECURE.load(Ordering::Relaxed)
}

pub fn unverified_client() -> reqwest::Client {
    reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .expect("failed to build HTTP client")
}

pub fn build_client() -> Result<reqwest::Client, Box<dyn std::error::Error>> {
    if is_insecure() {
        crate::display::warn("TLS verification disabled by --insecure");
        return Ok(unverified_client());
    }
    let builder = reqwest::Client::builder();
    let builder = match load_config().and_then(|config| config.ca_pem) {
        Some(pem) => builder.tls_certs_only([reqwest::Certificate::from_pem(pem.as_bytes())?]),
        None => builder,
    };
    Ok(builder.build()?)
}

pub fn auth() -> Result<(reqwest::Client, String, String), Box<dyn std::error::Error>> {
    let config = load_config().ok_or("not authenticated, run: csfx login")?;
    let token = config.token.ok_or("not authenticated, run: csfx login")?;
    if config.ca_pem.is_none() && !is_insecure() {
        return Err("no pinned cluster ca, run: csfx login".into());
    }
    Ok((build_client()?, config.server, token))
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

pub async fn get_text(
    client: &reqwest::Client,
    url: &str,
    token: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let resp = client
        .get(url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await?;
        crate::display::error(&format!("request failed: {} {}", status, body));
        std::process::exit(1);
    }

    Ok(resp.text().await?)
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

pub async fn request_json(
    client: &reqwest::Client,
    method: reqwest::Method,
    url: &str,
    token: &str,
    body: Option<&Value>,
) -> Result<Value, Box<dyn std::error::Error>> {
    let mut request = client
        .request(method, url)
        .header("Authorization", format!("Bearer {}", token));
    if let Some(body) = body {
        request = request.json(body);
    }
    let resp = request.send().await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let err = resp.text().await?;
        crate::display::error(&format!("request failed: {} {}", status, err));
        std::process::exit(1);
    }

    let text = resp.text().await?;
    if text.is_empty() {
        return Ok(Value::Null);
    }
    Ok(serde_json::from_str(&text)?)
}
