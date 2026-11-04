use crate::display::{self, Table};
use crate::http::{auth, base_url, get_json};
use serde::Deserialize;

#[derive(Deserialize)]
struct Group {
    id: String,
    name: String,
    internal_cidr: String,
    status: String,
}

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = auth()?;
    let url = format!("{}/resource-groups", base_url(&server));

    let pb = display::spinner("fetching resource groups...");
    let data = get_json(&client, &url, &token).await;
    pb.finish_and_clear();

    let groups: Vec<Group> = serde_json::from_value(data?)?;

    display::section("Resource groups");

    if groups.is_empty() {
        display::info("no resource groups found");
        return Ok(());
    }

    let mut table = Table::new(vec!["ID", "NAME", "NETWORK", "STATUS"]);
    for group in &groups {
        table.add_row(vec![
            group.id.clone(),
            group.name.clone(),
            group.internal_cidr.clone(),
            group.status.clone(),
        ]);
    }

    table.print();
    println!();
    display::info(&format!("{} resource group(s)", groups.len()));

    Ok(())
}
