use super::api::VaultApi;
use crate::display::{self, Table};
use crate::http::{get_json, request_json};
use reqwest::Method;
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
struct Grant {
    id: String,
    principal: String,
    capability: String,
    scope: String,
    requires_approval: bool,
}

pub fn capability_for(can: &str) -> Result<&'static str, Box<dyn std::error::Error>> {
    match can {
        "read" => Ok("read_secrets"),
        "reveal" => Ok("reveal_secrets"),
        "manage" => Ok("manage"),
        other => Err(format!("unknown access {}, use read, reveal or manage", other).into()),
    }
}

fn describe(grant: &Grant) -> (String, String) {
    let can = grant.capability.replace('_', " ");
    let on = match (grant.requires_approval, grant.scope.as_str()) {
        (true, _) => "needs approval".to_string(),
        (false, "*") => "all secrets".to_string(),
        (false, scope) => scope.to_string(),
    };
    (can, on)
}

pub async fn list(resource_group: &str, vault: &str) -> Result<(), Box<dyn std::error::Error>> {
    let api = VaultApi::connect()?;
    let id = api.resolve(resource_group, vault).await?;
    let url = format!("{}/grants", api.vault_url(resource_group, &id));
    let grants: Vec<Grant> =
        serde_json::from_value(get_json(&api.client, &url, &api.token).await?)?;

    display::section("Who can use what");
    if grants.is_empty() {
        display::info("no access granted yet, grant it with: csfx vault grants add");
        return Ok(());
    }
    let mut table = Table::new(vec!["ID", "WHO", "CAN", "ON"]);
    for grant in &grants {
        let (can, on) = describe(grant);
        table.add_row(vec![grant.id.clone(), grant.principal.clone(), can, on]);
    }
    table.print();
    Ok(())
}

pub async fn add(
    resource_group: &str,
    vault: &str,
    who: &str,
    can: &str,
    on: Option<String>,
    approval: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let capability = capability_for(can)?;
    let api = VaultApi::connect()?;
    let id = api.resolve(resource_group, vault).await?;
    let url = format!("{}/grants", api.vault_url(resource_group, &id));
    let body = json!({
        "principal": who,
        "capability": capability,
        "scope": on.unwrap_or_else(|| "*".to_string()),
        "requires_approval": approval,
    });
    request_json(&api.client, Method::POST, &url, &api.token, Some(&body)).await?;
    display::success(&format!("{} may now {}", who, can));
    Ok(())
}

pub async fn remove(
    resource_group: &str,
    vault: &str,
    grant_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let api = VaultApi::connect()?;
    let id = api.resolve(resource_group, vault).await?;
    let url = format!("{}/grants/{}", api.vault_url(resource_group, &id), grant_id);
    request_json(&api.client, Method::DELETE, &url, &api.token, None).await?;
    display::success("access revoked");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_access_words_to_capabilities() {
        assert_eq!(capability_for("read").unwrap(), "read_secrets");
        assert_eq!(capability_for("reveal").unwrap(), "reveal_secrets");
        assert_eq!(capability_for("manage").unwrap(), "manage");
        assert!(capability_for("write").is_err());
    }
}
