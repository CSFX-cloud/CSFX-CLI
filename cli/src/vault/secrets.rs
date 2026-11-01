use super::api::VaultApi;
use crate::display::{self, kv, Table};
use crate::http::{get_json, request_json};
use crate::user::login::{encrypt_password, get_public_key};
use dialoguer::{Input, Password};
use reqwest::Method;
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Deserialize)]
struct Secret {
    name: String,
    version: u64,
    updated_at: String,
    expires_at: Option<String>,
    deleted: bool,
}

fn secrets_url(api: &VaultApi, resource_group: &str, vault_id: &str) -> String {
    format!("{}/secrets", api.vault_url(resource_group, vault_id))
}

async fn fetch(
    api: &VaultApi,
    resource_group: &str,
    vault_id: &str,
) -> Result<Vec<Secret>, Box<dyn std::error::Error>> {
    let url = secrets_url(api, resource_group, vault_id);
    Ok(serde_json::from_value(
        get_json(&api.client, &url, &api.token).await?,
    )?)
}

pub async fn list(resource_group: &str, vault: &str) -> Result<(), Box<dyn std::error::Error>> {
    let api = VaultApi::connect()?;
    let id = api.resolve(resource_group, vault).await?;
    let pb = display::spinner("fetching secrets...");
    let secrets = fetch(&api, resource_group, &id).await;
    pb.finish_and_clear();
    let secrets = secrets?;

    display::section("Secrets");
    if secrets.is_empty() {
        display::info("no secrets yet, add one with: csfx vault secrets set");
        return Ok(());
    }
    let mut table = Table::new(vec!["NAME", "VERSION", "UPDATED", "EXPIRES", "STATE"]);
    for secret in &secrets {
        table.add_row(vec![
            secret.name.clone(),
            format!("v{}", secret.version),
            secret.updated_at.chars().take(10).collect(),
            secret.expires_at.clone().unwrap_or_else(|| "-".to_string()),
            if secret.deleted { "deleted" } else { "active" }.to_string(),
        ]);
    }
    table.print();
    println!();
    display::info("values stay hidden, show one with: csfx vault secrets reveal");
    Ok(())
}

pub async fn set(
    resource_group: &str,
    vault: &str,
    name: &str,
    value: Option<String>,
    expires: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let api = VaultApi::connect()?;
    let id = api.resolve(resource_group, vault).await?;
    let value = match value {
        Some(value) => value,
        None => Password::new().with_prompt("secret value").interact()?,
    };
    let exists = fetch(&api, resource_group, &id)
        .await?
        .iter()
        .any(|secret| secret.name == name);
    let body = json!({ "name": name, "value": value, "expires_at": expires });
    let (method, url) = if exists {
        (
            Method::PUT,
            format!("{}/{}", secrets_url(&api, resource_group, &id), name),
        )
    } else {
        (Method::POST, secrets_url(&api, resource_group, &id))
    };

    let pb = display::spinner("storing secret...");
    let reply = request_json(&api.client, method, &url, &api.token, Some(&body)).await;
    pb.finish_and_clear();

    let version = reply?["version"].as_u64().unwrap_or_default();
    display::success(&format!("{} stored as version {}", name, version));
    Ok(())
}

pub async fn delete(
    resource_group: &str,
    vault: &str,
    name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let api = VaultApi::connect()?;
    let id = api.resolve(resource_group, vault).await?;
    let url = format!("{}/{}", secrets_url(&api, resource_group, &id), name);
    request_json(&api.client, Method::DELETE, &url, &api.token, None).await?;
    display::success(&format!("{} deleted, it can be restored for 30 days", name));
    Ok(())
}

pub async fn restore(
    resource_group: &str,
    vault: &str,
    name: &str,
    version: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let api = VaultApi::connect()?;
    let id = api.resolve(resource_group, vault).await?;
    let url = format!(
        "{}/{}/restore",
        secrets_url(&api, resource_group, &id),
        name
    );
    request_json(
        &api.client,
        Method::POST,
        &url,
        &api.token,
        Some(&json!({ "version": version })),
    )
    .await?;
    display::success(&format!("{} restored", name));
    Ok(())
}

async fn proof(server_root: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let code: String = Input::new()
        .with_prompt("authenticator code, empty to use your password")
        .allow_empty(true)
        .interact_text()?;
    if !code.trim().is_empty() {
        return Ok(json!({ "code": code.trim() }));
    }
    let password = Password::new().with_prompt("password").interact()?;
    let key = get_public_key(server_root).await?;
    Ok(json!({ "password": encrypt_password(&password, &key)? }))
}

pub async fn reveal(
    resource_group: &str,
    vault: &str,
    name: &str,
    reason: &str,
    raw: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let api = VaultApi::connect()?;
    let id = api.resolve(resource_group, vault).await?;
    let mut body = proof(api.server()).await?;
    body["reason"] = json!(reason);
    let url = format!("{}/{}/reveal", secrets_url(&api, resource_group, &id), name);
    let reply = request_json(&api.client, Method::POST, &url, &api.token, Some(&body)).await?;

    let value = reply["value"].as_str().unwrap_or_default();
    if raw {
        println!("{}", value);
        return Ok(());
    }
    display::section(&format!("Secret  /  {}", name));
    kv("Version", &format!("v{}", reply["version"]));
    kv("Value", value);
    Ok(())
}
