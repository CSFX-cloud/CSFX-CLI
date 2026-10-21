use crate::display::{self, kv};
use crate::http::{auth, base_url, get_json};
use serde::Deserialize;

#[derive(Deserialize)]
struct Zone {
    zone: String,
    servers: Vec<String>,
    console_host: String,
}

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = auth()?;
    let url = format!("{}/dns/zone", base_url(&server));

    let pb = display::spinner("fetching dns zone...");
    let data = get_json(&client, &url, &token).await;
    pb.finish_and_clear();

    let zone: Zone = serde_json::from_value(data?)?;

    display::section("DNS  /  Platform Zone");
    kv("Zone", &zone.zone);
    kv("Servers", &zone.servers.join(", "));
    kv("Console", &format!("https://{}:8000", zone.console_host));
    println!();
    print_resolver_hint(&zone);

    Ok(())
}

fn print_resolver_hint(zone: &Zone) {
    let Some(server) = zone.servers.first() else {
        display::info("no dns server address is published, point your resolver at a master");
        return;
    };
    display::info("macOS: resolve the zone through the cluster");
    println!(
        "  printf 'nameserver {}\\n' | sudo tee /etc/resolver/{}",
        server, zone.zone
    );
    display::info("Linux with systemd-resolved");
    println!(
        "  sudo resolvectl dns <interface> {} && sudo resolvectl domain <interface> ~{}",
        server, zone.zone
    );
}
