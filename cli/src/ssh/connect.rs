use crate::display;
use crate::http::{auth, base_url, get_json};
use crate::ssh::access;
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
    name: String,
    hostname: String,
    ip_address: Option<String>,
    status: String,
    ssh_host_key: Option<String>,
}

struct Target {
    address: String,
    host_key: String,
}

#[derive(Deserialize)]
struct SshKeyRecord {
    id: String,
}

const SSH_LOGIN_USER: &str = "csfx";
const DEFAULT_SSH_PORT: u16 = 22;

pub async fn run(node: String, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = auth()?;
    let base = base_url(&server);

    if !access::fetch(&client, &base, &token).await?.effective {
        return Err("ssh access is disabled, enable it with: csfx ssh enable".into());
    }

    let target = resolve_node(&client, &base, &token, &node).await?;
    let target_ip = target.address.as_str();
    let username = SSH_LOGIN_USER;

    let tempdir = TempDir::new()?;
    let known_hosts_path = tempdir.path().join("known_hosts");
    let known_hosts = known_hosts_line(&target.address, port, &target.host_key)?;
    std::fs::write(&known_hosts_path, known_hosts)?;
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

    display::success(&format!(
        "connecting to {} ({}@{}:{})",
        node, username, target_ip, port
    ));

    let ssh_status = Command::new("ssh")
        .arg("-i")
        .arg(&private_key_path)
        .arg("-p")
        .arg(port.to_string())
        .arg("-o")
        .arg("StrictHostKeyChecking=yes")
        .arg("-o")
        .arg(format!("UserKnownHostsFile={}", known_hosts_path.display()))
        .arg("-o")
        .arg("GlobalKnownHostsFile=/dev/null")
        .arg("-o")
        .arg("HostKeyAlgorithms=ssh-ed25519")
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
) -> Result<Target, Box<dyn std::error::Error>> {
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
        display::warn(&format!(
            "node status is '{}', attempting connection anyway",
            agent.status
        ));
    }

    let host_key = agent
        .ssh_host_key
        .filter(|key| !key.is_empty())
        .ok_or("node has not reported an ssh host key, refusing to connect")?;
    let address = agent
        .ip_address
        .filter(|ip| !ip.is_empty())
        .unwrap_or_else(|| agent.hostname.clone());
    Ok(Target { address, host_key })
}

fn known_hosts_line(
    host: &str,
    port: u16,
    key: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut fields = key.split_whitespace();
    let (Some(key_type), Some(body), None) = (fields.next(), fields.next(), fields.next()) else {
        return Err("node reported a malformed ssh host key".into());
    };
    if key_type != "ssh-ed25519" || host.chars().any(|c| c.is_whitespace() || c == ',') {
        return Err("node reported an unsupported ssh host key".into());
    }
    let pattern = if port == DEFAULT_SSH_PORT {
        host.to_string()
    } else {
        format!("[{}]:{}", host, port)
    };
    Ok(format!("{} {} {}\n", pattern, key_type, body))
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

#[cfg(test)]
mod tests {
    use super::known_hosts_line;

    #[test]
    fn known_hosts_pin_the_reported_key_to_the_address() {
        assert_eq!(
            known_hosts_line("10.0.0.5", 22, "ssh-ed25519 AAAAC3").unwrap(),
            "10.0.0.5 ssh-ed25519 AAAAC3\n"
        );
        assert_eq!(
            known_hosts_line("10.0.0.5", 2222, "ssh-ed25519 AAAAC3").unwrap(),
            "[10.0.0.5]:2222 ssh-ed25519 AAAAC3\n"
        );
    }

    #[test]
    fn malformed_or_unsupported_keys_and_hosts_are_refused() {
        for key in [
            "",
            "ssh-ed25519",
            "ssh-rsa AAAA",
            "ssh-ed25519 AAAA extra",
            "ssh-ed25519 AAAA\n@revoked * ssh-ed25519 BBBB",
        ] {
            assert!(known_hosts_line("10.0.0.5", 22, key).is_err(), "{key:?}");
        }
        assert!(known_hosts_line("a b", 22, "ssh-ed25519 AAAA").is_err());
        assert!(known_hosts_line("a,b", 22, "ssh-ed25519 AAAA").is_err());
    }
}
