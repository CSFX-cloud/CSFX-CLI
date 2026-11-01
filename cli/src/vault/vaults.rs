use super::api::VaultApi;
use crate::display::{self, kv, Table};
use crate::http::{delete_req, get_json, post_json, request_json};
use reqwest::Method;
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
struct Vault {
    id: String,
    name: String,
    sealed: bool,
    created_at: String,
}

#[derive(Deserialize)]
struct Overview {
    #[serde(flatten)]
    vault: Vault,
    secret_count: u64,
    grant_count: u64,
    reads_24h: u64,
    denied_24h: u64,
}

fn state(vault: &Vault) -> &'static str {
    if vault.sealed {
        "sealed"
    } else {
        "unsealed"
    }
}

pub async fn list(resource_group: &str) -> Result<(), Box<dyn std::error::Error>> {
    let api = VaultApi::connect()?;
    let pb = display::spinner("fetching vaults...");
    let data = get_json(&api.client, &api.groups_url(resource_group), &api.token).await;
    pb.finish_and_clear();
    let vaults: Vec<Vault> = serde_json::from_value(data?)?;

    display::section("Key vaults");
    if vaults.is_empty() {
        display::info("no vaults found, create one with: csfx vault create");
        return Ok(());
    }
    let mut table = Table::new(vec!["ID", "NAME", "STATE", "CREATED"]);
    for vault in &vaults {
        table.add_row(vec![
            vault.id.clone(),
            vault.name.clone(),
            state(vault).to_string(),
            vault.created_at.chars().take(10).collect(),
        ]);
    }
    table.print();
    println!();
    display::info(&format!("{} vault(s)", vaults.len()));
    Ok(())
}

pub async fn create(resource_group: &str, name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let api = VaultApi::connect()?;
    let pb = display::spinner("creating vault...");
    let data = post_json(
        &api.client,
        &api.groups_url(resource_group),
        &api.token,
        &json!({ "name": name }),
    )
    .await;
    pb.finish_and_clear();
    let vault: Vault = serde_json::from_value(data?)?;

    display::section("Key vault  /  Created");
    kv("ID", &vault.id);
    kv("Name", &vault.name);
    kv("State", state(&vault));
    println!();
    display::info("add secrets with: csfx vault secrets set");
    Ok(())
}

pub async fn get(resource_group: &str, vault: &str) -> Result<(), Box<dyn std::error::Error>> {
    let api = VaultApi::connect()?;
    let id = api.resolve(resource_group, vault).await?;
    let data = get_json(&api.client, &api.vault_url(resource_group, &id), &api.token).await?;
    let overview: Overview = serde_json::from_value(data)?;

    display::section(&format!("Key vault  /  {}", overview.vault.name));
    kv("ID", &overview.vault.id);
    kv("State", state(&overview.vault));
    kv("Secrets", &overview.secret_count.to_string());
    kv("Access grants", &overview.grant_count.to_string());
    kv("Reads in 24 h", &overview.reads_24h.to_string());
    kv("Denied in 24 h", &overview.denied_24h.to_string());
    Ok(())
}

pub async fn delete(resource_group: &str, vault: &str) -> Result<(), Box<dyn std::error::Error>> {
    let api = VaultApi::connect()?;
    let id = api.resolve(resource_group, vault).await?;
    let pb = display::spinner("deleting vault and its secrets...");
    let result = delete_req(&api.client, &api.vault_url(resource_group, &id), &api.token).await;
    pb.finish_and_clear();
    result?;
    display::success(&format!("vault {} deleted", vault));
    Ok(())
}

pub async fn set_sealed(
    resource_group: &str,
    vault: &str,
    sealed: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let api = VaultApi::connect()?;
    let id = api.resolve(resource_group, vault).await?;
    let action = if sealed { "seal" } else { "unseal" };
    let url = format!("{}/{}", api.vault_url(resource_group, &id), action);
    request_json(&api.client, Method::POST, &url, &api.token, None).await?;
    display::success(&format!("vault {} {}ed", vault, action));
    Ok(())
}
