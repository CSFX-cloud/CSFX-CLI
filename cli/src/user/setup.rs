use dialoguer::{Input, Password};
use serde::{Deserialize, Serialize};

use crate::config::{load_config, save_config, Config};
use crate::display;
use crate::user::login::{encrypt_password, get_public_key};

const MIN_PASSWORD_LENGTH: usize = 12;
const MIN_PIN_LENGTH: usize = 8;

#[derive(Deserialize)]
struct SetupStatus {
    state: String,
    ca_fingerprint: Option<String>,
}

#[derive(Serialize)]
struct SetupRequest {
    setup_code: String,
    organization_name: String,
    admin_email: String,
    encrypted_password: String,
    encrypted_console_pin: String,
}

#[derive(Deserialize)]
struct SetupResponse {
    token: String,
    username: String,
    recovery_key: String,
}

fn prompt_secret(label: &str, minimum: usize) -> Result<String, Box<dyn std::error::Error>> {
    loop {
        let value = Password::new()
            .with_prompt(label)
            .with_confirmation("repeat", "entries do not match")
            .interact()?;
        if value.chars().count() >= minimum {
            return Ok(value);
        }
        display::warn(&format!("minimum length is {} characters", minimum));
    }
}

async fn fetch_status(server: &str) -> Result<SetupStatus, Box<dyn std::error::Error>> {
    let url = format!("{}/api/setup/status", server);
    let response = crate::http::build_client().get(&url).send().await?;
    if !response.status().is_success() {
        return Err(format!("setup status unavailable: {}", response.status()).into());
    }
    Ok(response.json().await?)
}

async fn submit(
    server: &str,
    request: &SetupRequest,
) -> Result<SetupResponse, Box<dyn std::error::Error>> {
    let url = format!("{}/api/setup", server);
    let response = crate::http::build_client()
        .post(&url)
        .json(request)
        .send()
        .await?;
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await?;
        return Err(format!("setup failed: {} {}", status, body).into());
    }
    Ok(response.json().await?)
}

fn store_session(server: &str, token: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut config = load_config().unwrap_or(Config {
        server: String::new(),
        token: None,
        compose_dir: None,
        ghcr_org: None,
    });
    config.server = server.to_string();
    config.token = Some(token.to_string());
    save_config(&config)
}

pub async fn setup(server: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let server: String = match server {
        Some(server) => server,
        None => Input::new()
            .with_prompt("server URL, e.g. https://192.168.1.10:8000")
            .interact_text()?,
    };
    let server = server.trim_end_matches('/').to_string();

    let status = fetch_status(&server).await?;
    if status.state != "setup_required" {
        return Err("setup already completed, run: csfx login".into());
    }
    display::kv(
        "CA fingerprint",
        status.ca_fingerprint.as_deref().unwrap_or("unavailable"),
    );
    display::info("compare the fingerprint with the node console before continuing");

    let setup_code: String = Input::new()
        .with_prompt("setup code shown on the node console")
        .interact_text()?;
    let organization_name: String = Input::new()
        .with_prompt("organization name")
        .interact_text()?;
    let admin_email: String = Input::new().with_prompt("admin email").interact_text()?;
    let password = prompt_secret("admin password (min 12 characters)", MIN_PASSWORD_LENGTH)?;
    let console_pin = prompt_secret("console PIN (min 8 characters)", MIN_PIN_LENGTH)?;

    let public_key = get_public_key(&server).await?;
    let request = SetupRequest {
        setup_code,
        organization_name,
        admin_email,
        encrypted_password: encrypt_password(&password, &public_key)?,
        encrypted_console_pin: encrypt_password(&console_pin, &public_key)?,
    };

    let pb = display::spinner("initializing cluster...");
    let response = submit(&server, &request).await;
    pb.finish_and_clear();
    let response = response?;

    store_session(&server, &response.token)?;
    display::success(&format!(
        "cluster initialized, logged in as {}",
        response.username
    ));
    display::warn("store this console recovery key offline, it is shown only once");
    display::kv("Recovery key", &response.recovery_key);
    Ok(())
}
