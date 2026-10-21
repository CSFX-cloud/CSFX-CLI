use crate::display::{self, Table};
use crate::http::{auth, base_url, get_json};
use serde::Deserialize;

#[derive(Deserialize)]
struct Route {
    id: String,
    hostname: String,
    workload_id: String,
    gateway_id: Option<String>,
    container_port: u16,
    tls: bool,
}

pub async fn run(resource_group: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = auth()?;
    let url = format!(
        "{}/resource-groups/{}/ingress",
        base_url(&server),
        resource_group
    );

    let pb = display::spinner("fetching ingress routes...");
    let data = get_json(&client, &url, &token).await;
    pb.finish_and_clear();

    let routes: Vec<Route> = serde_json::from_value(data?)?;

    display::section("Ingress");

    if routes.is_empty() {
        display::info("no ingress routes found");
        return Ok(());
    }

    let mut table = Table::new(vec!["ID", "HOSTNAME", "GATEWAY", "WORKLOAD", "PORT", "TLS"]);
    for route in &routes {
        table.add_row(vec![
            route.id.clone(),
            route.hostname.clone(),
            route
                .gateway_id
                .clone()
                .unwrap_or_else(|| "none".to_string()),
            route.workload_id.clone(),
            route.container_port.to_string(),
            if route.tls { "yes" } else { "no" }.to_string(),
        ]);
    }

    table.print();
    println!();
    display::info(&format!("{} route(s)", routes.len()));

    Ok(())
}
