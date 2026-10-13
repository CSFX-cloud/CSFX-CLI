use crate::display;
use crate::ssh::access;
use crate::http::{auth, base_url, get_json};
use chrono::{Duration, Utc};
use serde::Deserialize;
use serde_json::json;
use ssh_key::{Algorithm, LineEnding, PrivateKey};
use std::os::unix::fs::PermissionsExt;
use std::process::Command;
use tempfile::TempDir;
use uuid::Uuid;

#[derive(Deserialize)]
struct AgentInfo {
    id: String,
    name: String,
    hostname: String,
    ip_address: Option<String>,
    status: String,
}

#[derive(Deserialize)]
struct SshKeyRecord {
    id: String,
}

const SSH_LOGIN_USER: &str = "csfx";

pub async fn run(node: String, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = auth()?;
    let base = base_url(&server);

    if !access::fetch(&client, &base, &token).await?.effective {
        return Err("ssh access is disabled, enable it with: csfx ssh enable".into());
    }

    let target_ip = resolve_node(&client, &base, &token, &node).await?;
    let username = SSH_LOGIN_USER;

    let tempdir = TempDir::new()?;
    let private_key_path = tempdir.path().join("id_ed25519");
    let public_key_path = tempdir.path().join("id_ed25519.pub");

    let private_key = PrivateKey::random(&mut rand::rngs::OsRng, Algorithm::Ed25519)?;
    let public_key = private_key.public_key();

    private_key.write_openssh_file(&private_key_path, LineEnding::LF)?;
    std::fs::set_permissions(&private_key_path, std::fs::Permissions::from_mode(0o600))?;

    let pubkey_str = public_key.to_openssh()?;
    std::fs::write(&public_key_path, &pubkey_str)?;

    let expires_at = Utc::now() + Duration::minutes(5);
    let key_id = register_key(&client, &base, &token, &pubkey_str, expires_at).await?;

    display::success(&format!("connecting to {} ({}@{}:{})", node, username, target_ip, port));

    let ssh_status = Command::new("ssh")
        .arg("-i")
        .arg(&private_key_path)
        .arg("-p")
        .arg(port.to_string())
        .arg("-o")
        .arg("StrictHostKeyChecking=no")
        .arg("-o")
        .arg("UserKnownHostsFile=/dev/null")
        .arg("-o")
        .arg("LogLevel=ERROR")
        .arg(format!("{}@{}", username, target_ip))
        .status();

    delete_key(&client, &base, &token, &key_id).await;

    match ssh_status {
        Ok(status) if !status.success() => {
            if let Some(code) = status.code() {
                display::error(&format!("ssh exited with code {}", code));
            }
        }
        Err(e) => {
            display::error(&format!("failed to launch ssh: {}", e));
        }
        _ => {}
    }

    Ok(())
}

async fn resolve_node(
    client: &reqwest::Client,
    base: &str,
    token: &str,
    node: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let pb = display::spinner("resolving node...");

    let agent = if Uuid::parse_str(node).is_ok() {
        let url = format!("{}/agents/{}", base, node);
        let data = get_json(client, &url, token).await;
        pb.finish_and_clear();
        let data = data?;
        serde_json::from_value::<AgentInfo>(data)?
    } else {
        let url = format!("{}/agents", base);
        let data = get_json(client, &url, token).await;
        pb.finish_and_clear();
        let data = data?;
        let agents: Vec<AgentInfo> = serde_json::from_value(data)?;
        agents
            .into_iter()
            .find(|a| a.name == node || a.hostname == node)
            .ok_or_else(|| format!("node '{}' not found", node))?
    };

    if agent.status != "online" {
        display::warn(&format!("node status is '{}', attempting connection anyway", agent.status));
    }

    agent
        .ip_address
        .filter(|ip| !ip.is_empty())
        .or_else(|| Some(agent.hostname.clone()))
        .ok_or_else(|| "node has no reachable address".into())
}

async fn register_key(
    client: &reqwest::Client,
    base: &str,
    token: &str,
    public_key: &str,
    expires_at: chrono::DateTime<Utc>,
) -> Result<String, Box<dyn std::error::Error>> {
    let url = format!("{}/ssh-keys", base);
    let body = json!({
        "name": format!("csfx-ssh-session-{}", Utc::now().timestamp()),
        "public_key": public_key,
        "expires_at": expires_at.to_rfc3339(),
    });

    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("failed to register ssh key: {} {}", status, text).into());
    }

    let record: SshKeyRecord = resp.json().await?;
    Ok(record.id)
}

async fn delete_key(client: &reqwest::Client, base: &str, token: &str, key_id: &str) {
    let url = format!("{}/ssh-keys/{}", base, key_id);
    let _ = client
        .delete(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await;
}
