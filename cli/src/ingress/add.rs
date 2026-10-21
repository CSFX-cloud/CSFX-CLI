use crate::display::{self, kv};
use crate::http::{auth, base_url, post_json};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
struct Route {
    id: String,
    hostname: String,
    container_port: u16,
    tls: bool,
}

pub async fn run(
    resource_group: &str,
    hostname: &str,
    workload: &str,
    port: u16,
    gateway: &str,
    tls: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = auth()?;
    let url = format!(
        "{}/resource-groups/{}/ingress",
        base_url(&server),
        resource_group
    );
    let body = json!({
        "hostname": hostname,
        "gateway_id": gateway,
        "workload_id": workload,
        "container_port": port,
        "tls": tls,
    });

    let pb = display::spinner("creating ingress route...");
    let data = post_json(&client, &url, &token, &body).await;
    pb.finish_and_clear();

    let route: Route = serde_json::from_value(data?)?;

    display::section("Ingress  /  Route Created");
    kv("ID", &route.id);
    kv("Hostname", &route.hostname);
    kv("Port", &route.container_port.to_string());
    kv("TLS", if route.tls { "yes" } else { "no" });
    println!();
    display::info("hostnames inside the platform zone resolve automatically, see: csfx dns zone");
    display::info("for other hostnames point your DNS record at the agent that runs the gateway");

    Ok(())
}
