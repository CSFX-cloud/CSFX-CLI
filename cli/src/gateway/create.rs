use crate::display::{self, kv};
use crate::http::{auth, base_url, post_json};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
struct Gateway {
    id: String,
    name: String,
    http_port: u16,
    https_port: u16,
    endpoint: String,
}

pub async fn run(
    resource_group: &str,
    name: &str,
    http_port: u16,
    https_port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = auth()?;
    let url = format!(
        "{}/resource-groups/{}/gateways",
        base_url(&server),
        resource_group
    );
    let body = json!({
        "name": name,
        "http_port": http_port,
        "https_port": https_port,
    });

    let pb = display::spinner("creating gateway...");
    let data = post_json(&client, &url, &token, &body).await;
    pb.finish_and_clear();

    let gateway: Gateway = serde_json::from_value(data?)?;

    display::section("Gateway  /  Created");
    kv("ID", &gateway.id);
    kv("Name", &gateway.name);
    kv("HTTP port", &gateway.http_port.to_string());
    kv("HTTPS port", &gateway.https_port.to_string());
    kv("Endpoint", &gateway.endpoint);
    println!();
    display::info("the gateway starts as a workload, add routes with: csfx ingress add");

    Ok(())
}
