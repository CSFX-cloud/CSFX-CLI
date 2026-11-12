use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use dialoguer::Confirm;
use sha2::{Digest, Sha256};

use crate::config::{load_config, save_config, Config};
use crate::display;
use crate::http::{is_insecure, unverified_client};

pub fn fingerprint(pem: &str) -> Result<String, Box<dyn std::error::Error>> {
    let body: String = pem
        .lines()
        .skip_while(|line| !line.starts_with("-----BEGIN"))
        .skip(1)
        .take_while(|line| !line.starts_with("-----END"))
        .collect();
    if body.is_empty() {
        return Err("pem contains no certificate".into());
    }
    let der = BASE64.decode(body.trim())?;
    let hex: Vec<String> = Sha256::digest(der)
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect();
    Ok(hex.join(":"))
}

async fn download_ca(server: &str) -> Result<String, Box<dyn std::error::Error>> {
    let url = format!("{}/api/setup/ca.crt", server);
    let response = unverified_client()
        .get(&url)
        .send()
        .await?
        .error_for_status()?;
    let pem = response.text().await?;
    reqwest::Certificate::from_pem(pem.as_bytes())?;
    Ok(pem)
}

fn confirm_fingerprint(pem: &str) -> Result<(), Box<dyn std::error::Error>> {
    display::kv("CA fingerprint", &fingerprint(pem)?);
    display::info("compare the fingerprint with the node console before continuing");
    let trusted = Confirm::new()
        .with_prompt("trust this cluster ca")
        .default(false)
        .interact()?;
    if !trusted {
        return Err("cluster ca not trusted".into());
    }
    Ok(())
}

fn store_pin(server: &str, pem: String) -> Result<(), Box<dyn std::error::Error>> {
    let mut config = load_config().unwrap_or(Config {
        server: String::new(),
        token: None,
        compose_dir: None,
        ghcr_org: None,
        ca_pem: None,
    });
    if config.server != server {
        config.token = None;
    }
    config.server = server.to_string();
    config.ca_pem = Some(pem);
    save_config(&config)
}

pub async fn ensure_pinned(server: &str) -> Result<(), Box<dyn std::error::Error>> {
    let server = server.trim_end_matches('/');
    if is_insecure() {
        return Ok(());
    }
    let already_pinned =
        load_config().is_some_and(|config| config.server == server && config.ca_pem.is_some());
    if already_pinned {
        return Ok(());
    }
    let pem = download_ca(server).await?;
    confirm_fingerprint(&pem)?;
    store_pin(server, pem)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fingerprint_hashes_the_der_body() {
        let pem = "-----BEGIN CERTIFICATE-----\nAAEC\n-----END CERTIFICATE-----\n";
        assert_eq!(
            fingerprint(pem).unwrap(),
            "AE:4B:32:80:E5:6E:2F:AF:83:F4:14:A6:E3:DA:BE:9D:5F:BE:18:97:65:44:C0:5F:ED:12:1A:CC:B8:5B:53:FC"
        );
    }

    #[test]
    fn fingerprint_rejects_garbage() {
        assert!(fingerprint("not a certificate").is_err());
    }
}
