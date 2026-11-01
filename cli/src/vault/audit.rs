use super::api::VaultApi;
use crate::display::{self, Table};
use crate::http::get_json;
use colored::Color;
use serde::Deserialize;

#[derive(Deserialize)]
struct Entry {
    occurred_at: String,
    actor: String,
    action: String,
    source: String,
    result: String,
}

pub async fn run(resource_group: &str, vault: &str) -> Result<(), Box<dyn std::error::Error>> {
    let api = VaultApi::connect()?;
    let id = api.resolve(resource_group, vault).await?;
    let url = format!("{}/audit", api.vault_url(resource_group, &id));
    let entries: Vec<Entry> =
        serde_json::from_value(get_json(&api.client, &url, &api.token).await?)?;

    display::section("Audit log");
    if entries.is_empty() {
        display::info("nothing recorded yet");
        return Ok(());
    }
    let mut table =
        Table::new(vec!["TIME", "WHO", "ACTION", "FROM", "RESULT"]).with_color(|column, cell| {
            if column == 4 && cell.starts_with("denied") {
                Color::Red
            } else {
                Color::White
            }
        });
    for entry in &entries {
        table.add_row(vec![
            entry
                .occurred_at
                .chars()
                .take(19)
                .collect::<String>()
                .replace('T', " "),
            entry.actor.clone(),
            entry.action.clone(),
            entry.source.clone(),
            entry.result.clone(),
        ]);
    }
    table.print();
    Ok(())
}
