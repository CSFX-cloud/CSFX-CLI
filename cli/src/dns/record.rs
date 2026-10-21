use crate::display::{self, kv, Table};
use crate::http::{auth, base_url, delete_req, get_json, post_json};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
struct Record {
    id: String,
    name: String,
    record_type: String,
    value: String,
    ttl: i32,
}

fn records_url(server: &str, resource_group: &str) -> String {
    format!(
        "{}/resource-groups/{}/dns-records",
        base_url(server),
        resource_group
    )
}

pub async fn list(resource_group: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = auth()?;

    let pb = display::spinner("fetching dns records...");
    let data = get_json(&client, &records_url(&server, resource_group), &token).await;
    pb.finish_and_clear();

    let records: Vec<Record> = serde_json::from_value(data?)?;

    display::section("DNS  /  Records");

    if records.is_empty() {
        display::info("no manual dns records found");
        return Ok(());
    }

    let mut table = Table::new(vec!["ID", "NAME", "TYPE", "VALUE", "TTL"]);
    for record in &records {
        table.add_row(vec![
            record.id.clone(),
            record.name.clone(),
            record.record_type.clone(),
            record.value.clone(),
            record.ttl.to_string(),
        ]);
    }

    table.print();
    println!();
    display::info(&format!("{} record(s)", records.len()));

    Ok(())
}

pub async fn add(
    resource_group: &str,
    name: &str,
    record_type: &str,
    value: &str,
    ttl: Option<i32>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = auth()?;
    let body = json!({
        "name": name,
        "record_type": record_type,
        "value": value,
        "ttl": ttl,
    });

    let pb = display::spinner("creating dns record...");
    let data = post_json(
        &client,
        &records_url(&server, resource_group),
        &token,
        &body,
    )
    .await;
    pb.finish_and_clear();

    let record: Record = serde_json::from_value(data?)?;

    display::section("DNS  /  Record Created");
    kv("ID", &record.id);
    kv("Name", &record.name);
    kv("Type", &record.record_type);
    kv("Value", &record.value);
    kv("TTL", &record.ttl.to_string());

    Ok(())
}

pub async fn delete(resource_group: &str, id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = auth()?;
    let url = format!("{}/{}", records_url(&server, resource_group), id);

    let pb = display::spinner("deleting dns record...");
    let result = delete_req(&client, &url, &token).await;
    pb.finish_and_clear();

    result?;
    display::success(&format!("dns record {} deleted", id));

    Ok(())
}
