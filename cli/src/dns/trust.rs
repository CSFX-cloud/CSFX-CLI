use crate::config::load_config;
use crate::display::{self, kv};
use crate::http::{base_url, build_client};

const CA_FILE: &str = "csfx-ca.crt";

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config().ok_or("not configured, run: csfx login")?;
    let url = format!("{}/setup/ca.crt", base_url(&config.server));

    let pb = display::spinner("downloading cluster ca...");
    let response = build_client().get(&url).send().await;
    pb.finish_and_clear();

    let response = response?.error_for_status()?;
    let pem = response.text().await?;
    std::fs::write(CA_FILE, pem)?;

    display::section("DNS  /  Cluster CA");
    kv("File", CA_FILE);
    println!();
    display::info("macOS");
    println!(
        "  sudo security add-trusted-cert -d -r trustRoot -k /Library/Keychains/System.keychain {}",
        CA_FILE
    );
    display::info("Debian and Ubuntu");
    println!(
        "  sudo cp {} /usr/local/share/ca-certificates/ && sudo update-ca-certificates",
        CA_FILE
    );

    Ok(())
}
