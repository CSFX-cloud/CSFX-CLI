use crate::display::{self, Table};
use crate::http::{auth, base_url, get_json, post_json};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
struct Snapshot {
    id: String,
    volume_id: String,
    name: String,
    status: String,
    created_at: String,
}

pub async fn list(volume_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = auth()?;
    let url = format!("{}/volumes/{}/snapshots", base_url(&server), volume_id);

    let pb = display::spinner("fetching snapshots...");
    let data = get_json(&client, &url, &token).await;
    pb.finish_and_clear();

    let snapshots: Vec<Snapshot> = serde_json::from_value(data?)?;

    display::section("Snapshots");

    if snapshots.is_empty() {
        display::info("no snapshots found");
        return Ok(());
    }

    let mut table =
        Table::new(vec!["ID", "NAME", "VOLUME", "STATUS", "CREATED"]).with_color(|col, val| {
            if col == 3 {
                display::status_color(val)
            } else {
                colored::Color::White
            }
        });

    for s in &snapshots {
        table.add_row(vec![
            s.id[..8].to_string(),
            s.name.clone(),
            s.volume_id[..8].to_string(),
            s.status.clone(),
            s.created_at.clone(),
        ]);
    }

    table.print();
    println!();
    display::info(&format!("{} snapshot(s)", snapshots.len()));

    Ok(())
}

pub async fn create(volume_id: &str, name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = auth()?;
    let url = format!("{}/volumes/{}/snapshots", base_url(&server), volume_id);

    let body = json!({ "name": name });

    let pb = display::spinner("creating snapshot...");
    let data = post_json(&client, &url, &token, &body).await;
    pb.finish_and_clear();

    let snap: Snapshot = serde_json::from_value(data?)?;

    display::section("Snapshot Created");
    crate::display::kv("ID", &snap.id);
    crate::display::kv("Name", &snap.name);
    crate::display::kv("Volume", &snap.volume_id);
    crate::display::kv("Status", &snap.status);
    crate::display::kv("Created", &snap.created_at);
    println!();
    display::success("snapshot created");

    Ok(())
}
