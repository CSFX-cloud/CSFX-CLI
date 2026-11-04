use crate::http::{auth, base_url, get_json};
use serde::Deserialize;
use std::net::Ipv4Addr;

const DNS_HOST_OFFSET: u32 = 2;

#[derive(Deserialize)]
pub struct Group {
    pub id: String,
    pub name: String,
    pub internal_cidr: String,
}

#[derive(Deserialize)]
struct Zone {
    zone: String,
}

pub async fn find_group(name: &str) -> Result<Group, Box<dyn std::error::Error>> {
    let (client, server, token) = auth()?;
    let url = format!("{}/resource-groups", base_url(&server));
    let groups: Vec<Group> = serde_json::from_value(get_json(&client, &url, &token).await?)?;
    groups
        .into_iter()
        .find(|group| group.name.eq_ignore_ascii_case(name) || group.id == name)
        .ok_or_else(|| Box::<dyn std::error::Error>::from(format!("resource group not found name={}", name)))
}

pub async fn platform_zone() -> Result<String, Box<dyn std::error::Error>> {
    let (client, server, token) = auth()?;
    let url = format!("{}/dns/zone", base_url(&server));
    let zone: Zone = serde_json::from_value(get_json(&client, &url, &token).await?)?;
    Ok(zone.zone)
}

pub fn dns_address(cidr: &str) -> Option<Ipv4Addr> {
    let (network, _) = cidr.split_once('/')?;
    let network: Ipv4Addr = network.parse().ok()?;
    Some(Ipv4Addr::from(u32::from(network) + DNS_HOST_OFFSET))
}

#[cfg(test)]
mod tests {
    use super::dns_address;

    #[test]
    fn dns_runs_on_the_second_host_of_the_group() {
        assert_eq!(dns_address("10.20.0.0/24"), Some("10.20.0.2".parse().unwrap()));
    }

    #[test]
    fn malformed_cidr_has_no_dns_address() {
        assert_eq!(dns_address("10.20.0.0"), None);
        assert_eq!(dns_address("nonsense/24"), None);
    }
}
