use crate::display::{self, Table};
use crate::http::{auth, base_url, get_json};
use serde::Deserialize;

#[derive(Deserialize)]
struct Gateway {
    id: String,
    name: String,
    http_port: u16,
    https_port: u16,
    endpoint: String,
    status: Option<String>,
}

pub async fn run(resource_group: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = auth()?;
    let url = format!(
        "{}/resource-groups/{}/gateways",
        base_url(&server),
        resource_group
    );

    let pb = display::spinner("fetching gateways...");
    let data = get_json(&client, &url, &token).await;
    pb.finish_and_clear();

    let gateways: Vec<Gateway> = serde_json::from_value(data?)?;

    display::section("Gateways");

    if gateways.is_empty() {
        display::info("no gateways found, create one with: csfx gateway create");
        return Ok(());
    }

    let mut table = Table::new(vec!["ID", "NAME", "STATUS", "HTTP", "HTTPS", "ENDPOINT"]);
    for gateway in &gateways {
        table.add_row(vec![
            gateway.id.clone(),
            gateway.name.clone(),
            gateway
                .status
                .clone()
                .unwrap_or_else(|| "unknown".to_string()),
            gateway.http_port.to_string(),
            gateway.https_port.to_string(),
            gateway.endpoint.clone(),
        ]);
    }

    table.print();
    println!();
    display::info(&format!("{} gateway(s)", gateways.len()));

    Ok(())
}
