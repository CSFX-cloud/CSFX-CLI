use crate::display::{self, Table};
use crate::http::{auth, base_url, get_json};
use serde::Deserialize;

#[derive(Deserialize)]
struct Volume {
    id: String,
    name: String,
    size_gb: i32,
    pool: String,
    status: String,
    attached_to_agent: Option<String>,
}

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = auth()?;
    let url = format!("{}/volumes", base_url(&server));

    let pb = display::spinner("fetching volumes...");
    let data = get_json(&client, &url, &token).await;
    pb.finish_and_clear();

    let volumes: Vec<Volume> = serde_json::from_value(data?)?;

    display::section("Volumes");

    if volumes.is_empty() {
        display::info("no volumes found");
        return Ok(());
    }

    let mut table =
        Table::new(vec!["ID", "NAME", "SIZE", "POOL", "STATUS", "AGENT"]).with_color(|col, val| {
            if col == 4 {
                display::status_color(val)
            } else {
                colored::Color::White
            }
        });

    for v in &volumes {
        table.add_row(vec![
            v.id[..8].to_string(),
            v.name.clone(),
            format!("{}G", v.size_gb),
            v.pool.clone(),
            v.status.clone(),
            v.attached_to_agent
                .as_deref()
                .map(|s| s[..8].to_string())
                .unwrap_or_else(|| "-".to_string()),
        ]);
    }

    table.print();
    println!();
    display::info(&format!("{} volume(s)", volumes.len()));

    Ok(())
}
