use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use dialoguer::{Input, Password};
use rsa::pkcs1::DecodeRsaPublicKey;
use rsa::{Oaep, RsaPublicKey};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

use crate::config::{load_config, save_config, Config};
use crate::display;

#[derive(Serialize, Deserialize)]
struct LoginRequest {
    username: String,
    encrypted_password: String,
    two_factor_code: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct LoginResponse {
    token: String,
    user_id: String,
    username: String,
    two_factor_enabled: bool,
    force_password_change: bool,
}

#[derive(Serialize, Deserialize)]
struct PublicKeyResponse {
    public_key: String,
}

#[derive(Serialize, Deserialize)]
struct ChangePasswordRequest {
    old_password: String,
    new_password: String,
}

pub(crate) async fn get_public_key(server: &str) -> Result<RsaPublicKey, Box<dyn std::error::Error>> {
    let client = crate::http::build_client();
    let url = format!("{}/api/public-key", server.trim_end_matches('/'));

    let response = client
        .get(&url)
        .header("accept", "application/json")
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("failed to fetch public key: {}", response.status()).into());
    }

    let text = response.text().await?;
    let key_response: PublicKeyResponse = serde_json::from_str(&text)?;
    let public_key = RsaPublicKey::from_pkcs1_pem(&key_response.public_key)?;

    Ok(public_key)
}

pub(crate) fn encrypt_password(
    password: &str,
    public_key: &RsaPublicKey,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut rng = rand::thread_rng();
    let padding = Oaep::new::<Sha256>();
    let encrypted = public_key.encrypt(&mut rng, padding, password.as_bytes())?;
    Ok(BASE64.encode(&encrypted))
}

fn prompt_new_password() -> Result<String, Box<dyn std::error::Error>> {
    loop {
        let new_password = Password::new().with_prompt("new password").interact()?;
        let confirm = Password::new().with_prompt("confirm password").interact()?;

        if new_password == confirm {
            return Ok(new_password);
        }

        display::warn("passwords do not match, try again");
    }
}

async fn change_password(
    server: &str,
    token: &str,
    public_key: &RsaPublicKey,
    old_password_plain: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    display::warn("password change required");
    let new_password = prompt_new_password()?;
    let encrypted_old = encrypt_password(old_password_plain, public_key)?;
    let encrypted_new = encrypt_password(&new_password, public_key)?;

    let client = crate::http::build_client();
    let url = format!("{}/api/change-password", server.trim_end_matches('/'));

    let response = client
        .post(&url)
        .header("accept", "application/json")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .json(&ChangePasswordRequest {
            old_password: encrypted_old,
            new_password: encrypted_new,
        })
        .send()
        .await?;

    if response.status().is_success() {
        display::success("password changed");
        Ok(())
    } else {
        let status = response.status();
        let body = response.text().await?;
        Err(format!("password change failed: {} - {}", status, body).into())
    }
}

pub async fn login() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = load_config().unwrap_or_else(|| Config {
        server: String::new(),
        token: None,
        compose_dir: None,
        ghcr_org: None,
    });

    let server: String = if config.server.is_empty() {
        Input::new()
            .with_prompt("server URL")
            .interact_text()?
    } else {
        Input::new()
            .with_prompt("server URL")
            .default(config.server.clone())
            .interact_text()?
    };

    let pb = display::spinner("connecting to server...");
    let public_key = get_public_key(&server).await;
    pb.finish_and_clear();
    let public_key = public_key?;

    let username: String = Input::new().with_prompt("username").interact_text()?;
    let password = Password::new().with_prompt("password").interact()?;
    let encrypted_password = encrypt_password(&password, &public_key)?;

    let pb = display::spinner("authenticating...");

    let client = crate::http::build_client();
    let login_url = format!("{}/api/login", server.trim_end_matches('/'));

    let response = client
        .post(&login_url)
        .header("accept", "application/json")
        .header("Content-Type", "application/json")
        .json(&LoginRequest {
            username: username.clone(),
            encrypted_password: encrypted_password.clone(),
            two_factor_code: String::new(),
        })
        .send()
        .await?;

    pb.finish_and_clear();

    let response = if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await?;

        if error_text.contains("2FA")
            || error_text.contains("two-factor")
            || error_text.contains("two_factor")
        {
            let code: String = Input::new().with_prompt("2FA code").interact_text()?;

            let pb = display::spinner("authenticating with 2FA...");
            let resp = client
                .post(&login_url)
                .header("accept", "application/json")
                .header("Content-Type", "application/json")
                .json(&LoginRequest {
                    username: username.clone(),
                    encrypted_password: encrypted_password.clone(),
                    two_factor_code: code,
                })
                .send()
                .await?;
            pb.finish_and_clear();
            resp
        } else {
            display::error(&format!("login failed: {} - {}", status, error_text));
            std::process::exit(1);
        }
    } else {
        response
    };

    if response.status().is_success() {
        let text = response.text().await?;
        let login_response: LoginResponse = serde_json::from_str(&text)?;

        config.server = server.clone();
        config.token = Some(login_response.token.clone());
        save_config(&config)?;

        display::success(&format!("logged in as {}", login_response.username));
        display::kv("User ID", &login_response.user_id);
        display::kv(
            "2FA",
            if login_response.two_factor_enabled {
                "enabled"
            } else {
                "disabled"
            },
        );

        if login_response.force_password_change {
            change_password(&server, &login_response.token, &public_key, &password).await?;
        }
    } else {
        let status = response.status();
        let body = response.text().await?;
        display::error(&format!("login failed: {} - {}", status, body));
        std::process::exit(1);
    }

    Ok(())
}
