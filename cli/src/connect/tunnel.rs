use super::privileged;
use super::resolve::{dns_address, platform_zone, Group};
use super::resolver;
use crate::config::{config_dir, create_private_dir, ensure_private_dir, write_private};
use crate::display;
use crate::http::{auth, base_url, get_text};
use std::fs;
use std::path::{Path, PathBuf};

const WG_QUICK: &str = "wg-quick";
const INSTALL_HINT: &str = if cfg!(target_os = "macos") {
    "brew install wireguard-tools"
} else {
    "apt install wireguard-tools"
};
const INTERFACE_ID_LENGTH: usize = 8;

type Failure = Box<dyn std::error::Error>;

fn interface_name(group: &Group) -> String {
    let id: String = group.id.chars().take(INTERFACE_ID_LENGTH).collect();
    format!("csfx-{}", id)
}

fn config_path(group: &Group) -> Result<PathBuf, Failure> {
    ensure_private_dir()?;
    let directory = config_dir().join("vpn");
    create_private_dir(&directory)?;
    Ok(directory.join(format!("{}.conf", interface_name(group))))
}

fn find_tool(name: &str) -> Result<PathBuf, Failure> {
    let search = std::env::var_os("PATH").ok_or("PATH is not set")?;
    std::env::split_paths(&search)
        .map(|directory| directory.join(name))
        .find(|candidate| candidate.is_file())
        .ok_or_else(|| {
            Failure::from(format!(
                "{} not found, install it with: {}",
                name, INSTALL_HINT
            ))
        })
}

const INTERFACE_KEYS: [&str; 5] = ["privatekey", "address", "listenport", "mtu", "dns"];
const PEER_KEYS: [&str; 5] = [
    "publickey",
    "presharedkey",
    "endpoint",
    "allowedips",
    "persistentkeepalive",
];

fn value_is_plain(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "+/=,.:[]_- ".contains(c))
}

fn allowed_keys(section: &str) -> &'static [&'static str] {
    match section {
        "[interface]" => &INTERFACE_KEYS,
        "[peer]" => &PEER_KEYS,
        _ => &[],
    }
}

fn validated(config: &str) -> Result<String, Failure> {
    let mut section = String::new();
    let mut output = String::new();
    for line in config.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') {
            section = line.to_ascii_lowercase();
            if allowed_keys(&section).is_empty() {
                return Err(format!("wireguard section not allowed: {}", line).into());
            }
            output.push_str(&format!("{}\n", line));
            continue;
        }
        let (key, value) = line
            .split_once('=')
            .map(|(key, value)| (key.trim(), value.trim()))
            .ok_or_else(|| Failure::from("wireguard configuration line is malformed"))?;
        if !allowed_keys(&section).contains(&key.to_ascii_lowercase().as_str()) {
            return Err(format!("wireguard option not allowed: {}", key).into());
        }
        if !value_is_plain(value) {
            return Err(format!("wireguard value not allowed for {}", key).into());
        }
        output.push_str(&format!("{} = {}\n", key, value));
    }
    Ok(output)
}

fn verified_cache(path: &Path) -> Result<(), Failure> {
    let cached = fs::read_to_string(path)?;
    validated(&cached)
        .map(|_| ())
        .map_err(|error| format!("{}, rerun with --refresh", error).into())
}

fn without_dns(config: &str) -> String {
    config
        .lines()
        .filter(|line| !line.trim_start().starts_with("DNS"))
        .map(|line| format!("{}\n", line))
        .collect()
}

async fn ensure_config(group: &Group, refresh: bool) -> Result<PathBuf, Failure> {
    let path = config_path(group)?;
    if path.exists() && !refresh {
        verified_cache(&path)?;
        return Ok(path);
    }
    let (client, server, token) = auth()?;
    let url = format!(
        "{}/resource-groups/{}/vpn-config",
        base_url(&server),
        group.id
    );
    let spinner = display::spinner("requesting wireguard configuration...");
    let config = get_text(&client, &url, &token).await;
    spinner.finish_and_clear();
    let config = validated(&config?)?;
    let scoped = if cfg!(target_os = "macos") {
        without_dns(&config)
    } else {
        config
    };
    write_private(&path, &scoped)?;
    Ok(path)
}

fn wg_quick_command(tool: &Path, action: &str, config: &Path) -> Vec<String> {
    vec![
        "env".to_string(),
        format!("PATH={}", std::env::var("PATH").unwrap_or_default()),
        tool.display().to_string(),
        action.to_string(),
        config.display().to_string(),
    ]
}

fn switch_interface(
    tool: &Path,
    action: &str,
    config: &Path,
    tolerated: &str,
) -> Result<(), Failure> {
    let output = privileged::output(wg_quick_command(tool, action, config))?;
    if output.status.success() {
        return Ok(());
    }
    let message = String::from_utf8_lossy(&output.stderr).to_string();
    if message.contains(tolerated) {
        display::info(&format!("tunnel unchanged action={}", action));
        return Ok(());
    }
    Err(format!("wg-quick {} failed: {}", action, message.trim()).into())
}

pub async fn up(group: &Group, refresh: bool) -> Result<(), Failure> {
    let tool = find_tool(WG_QUICK)?;
    let config = ensure_config(group, refresh).await?;
    let zone = platform_zone().await?;
    if refresh {
        switch_interface(&tool, "down", &config, "is not a WireGuard interface")?;
    }
    switch_interface(&tool, "up", &config, "already exists")?;
    resolver::install(group, &zone)?;
    print_summary(group, &zone);
    Ok(())
}

pub async fn down(group: &Group) -> Result<(), Failure> {
    let tool = find_tool(WG_QUICK)?;
    let config = config_path(group)?;
    if config.exists() {
        verified_cache(&config)?;
    }
    let zone = platform_zone().await?;
    switch_interface(&tool, "down", &config, "is not a WireGuard interface")?;
    resolver::remove(group, &zone)?;
    display::success(&format!("disconnected resource group={}", group.name));
    Ok(())
}

fn print_summary(group: &Group, zone: &str) {
    display::success(&format!("connected resource group={}", group.name));
    display::kv("Interface", &interface_name(group));
    display::kv("Network", &group.internal_cidr);
    if let Some(address) = dns_address(&group.internal_cidr) {
        display::kv("DNS", &address.to_string());
    }
    display::kv("Domains", &resolver::domains(group, zone).join(", "));
    display::info(&format!(
        "leave with: csfx connect rg {} --down",
        group.name
    ));
}

#[cfg(test)]
mod tests {
    use super::{validated, without_dns};

    const GOOD: &str = "[Interface]\nPrivateKey = a+b/c=\nAddress = 10.20.0.254/32\nDNS = 10.20.0.2, svc.x.internal\n\n[Peer]\nPublicKey = k=\nEndpoint = 192.0.2.1:51820\nAllowedIPs = 10.20.0.0/24\nPersistentKeepalive = 25\n";

    #[test]
    fn a_plain_configuration_passes_unchanged() {
        assert_eq!(validated(GOOD).unwrap(), GOOD.replace("\n\n", "\n"));
    }

    #[test]
    fn hooks_and_routing_options_are_rejected() {
        for key in [
            "PostUp",
            "preup",
            "PreDown",
            "PostDown",
            "SaveConfig",
            "Table",
            "FwMark",
        ] {
            let config = GOOD.replace("[Peer]", &format!("{} = touch /tmp/x\n[Peer]", key));
            assert!(validated(&config).is_err(), "{key}");
        }
    }

    #[test]
    fn shell_metacharacters_and_unknown_sections_are_rejected() {
        let injected = GOOD.replace("10.20.0.254/32", "10.20.0.254/32; touch /tmp/x");
        assert!(validated(&injected).is_err());
        assert!(validated("[Script]\nPostUp = x\n").is_err());
        assert!(validated("PostUp = touch /tmp/x\n").is_err());
        assert!(validated("[Peer]\nPublicKey = $(id)\n").is_err());
    }

    #[test]
    fn dns_line_is_removed_and_everything_else_kept() {
        let config = "[Interface]\nPrivateKey = k\nDNS = 10.20.0.2, svc.x.internal\nAddress = 10.20.0.254/32\n";
        assert_eq!(
            without_dns(config),
            "[Interface]\nPrivateKey = k\nAddress = 10.20.0.254/32\n"
        );
    }
}
