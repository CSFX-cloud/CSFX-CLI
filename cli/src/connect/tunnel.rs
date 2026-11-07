use super::privileged;
use super::resolve::{dns_address, platform_zone, Group};
use super::resolver;
use crate::config::get_config_path;
use crate::display;
use crate::http::{auth, base_url, get_text};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

const WG_QUICK: &str = "wg-quick";
const INSTALL_HINT: &str = if cfg!(target_os = "macos") {
    "brew install wireguard-tools"
} else {
    "apt install wireguard-tools"
};
const INTERFACE_ID_LENGTH: usize = 8;
const CONFIG_MODE: u32 = 0o600;

type Failure = Box<dyn std::error::Error>;

fn interface_name(group: &Group) -> String {
    let id: String = group.id.chars().take(INTERFACE_ID_LENGTH).collect();
    format!("csfx-{}", id)
}

fn config_path(group: &Group) -> Result<PathBuf, Failure> {
    let mut directory = get_config_path();
    directory.pop();
    directory.push("vpn");
    fs::create_dir_all(&directory)?;
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

fn without_dns(config: &str) -> String {
    config
        .lines()
        .filter(|line| !line.trim_start().starts_with("DNS"))
        .map(|line| format!("{}\n", line))
        .collect()
}

fn write_private(path: &Path, content: &str) -> Result<(), Failure> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(CONFIG_MODE)
        .open(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

async fn ensure_config(group: &Group, refresh: bool) -> Result<PathBuf, Failure> {
    let path = config_path(group)?;
    if path.exists() && !refresh {
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
    let config = config?;
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
    use super::without_dns;

    #[test]
    fn dns_line_is_removed_and_everything_else_kept() {
        let config = "[Interface]\nPrivateKey = k\nDNS = 10.20.0.2, svc.x.internal\nAddress = 10.20.0.254/32\n";
        assert_eq!(
            without_dns(config),
            "[Interface]\nPrivateKey = k\nAddress = 10.20.0.254/32\n"
        );
    }
}
