use crate::http::{auth, base_url, get_json};
use reqwest::Client;
use serde_json::Value;

pub struct VaultApi {
    pub client: Client,
    pub token: String,
    server: String,
    root: String,
}

impl VaultApi {
    pub fn connect() -> Result<Self, Box<dyn std::error::Error>> {
        let (client, server, token) = auth()?;
        Ok(Self {
            client,
            token,
            root: base_url(&server),
            server,
        })
    }

    pub fn server(&self) -> &str {
        &self.server
    }

    pub fn groups_url(&self, resource_group: &str) -> String {
        format!("{}/resource-groups/{}/vaults", self.root, resource_group)
    }

    pub fn vault_url(&self, resource_group: &str, vault_id: &str) -> String {
        format!("{}/{}", self.groups_url(resource_group), vault_id)
    }

    pub async fn resolve(
        &self,
        resource_group: &str,
        vault: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        if uuid::Uuid::parse_str(vault).is_ok() {
            return Ok(vault.to_string());
        }
        let listed = get_json(&self.client, &self.groups_url(resource_group), &self.token).await?;
        find_vault_id(&listed, vault).ok_or_else(|| format!("vault not found: {}", vault).into())
    }
}

pub fn find_vault_id(listed: &Value, name: &str) -> Option<String> {
    listed
        .as_array()?
        .iter()
        .find(|vault| vault["name"] == name)
        .and_then(|vault| vault["id"].as_str())
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn finds_a_vault_by_name() {
        let listed = json!([{"id": "a", "name": "kv-one"}, {"id": "b", "name": "kv-two"}]);
        assert_eq!(find_vault_id(&listed, "kv-two"), Some("b".to_string()));
        assert_eq!(find_vault_id(&listed, "missing"), None);
        assert_eq!(find_vault_id(&json!({}), "kv-one"), None);
    }
}
