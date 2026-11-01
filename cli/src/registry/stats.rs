use crate::display::{self, section};
use crate::http::{auth, base_url, get_json};
use colored::Colorize;
use serde::Deserialize;

#[derive(Deserialize)]
struct Statistics {
    total: u64,
    online: u64,
    offline: u64,
    degraded: u64,
}

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = auth()?;
    let url = format!("{}/registry/admin/statistics", base_url(&server));

    let pb = display::spinner("fetching registry statistics...");
    let data = get_json(&client, &url, &token).await;
    pb.finish_and_clear();

    let data = data?;
    let s: Statistics = serde_json::from_value(data)?;

    section("Registry  /  Statistics");

    println!("  {:<20} {}", "Total".dimmed(), s.total.to_string().bold());
    println!(
        "  {:<20} {}",
        "Online".dimmed(),
        s.online.to_string().green().bold()
    );
    println!(
        "  {:<20} {}",
        "Offline".dimmed(),
        s.offline.to_string().red().bold()
    );
    println!(
        "  {:<20} {}",
        "Degraded".dimmed(),
        s.degraded.to_string().yellow().bold()
    );

    println!();

    let health_pct = if s.total > 0 {
        (s.online as f64 / s.total as f64) * 100.0
    } else {
        0.0
    };

    let bar_width = 40usize;
    let filled = ((health_pct / 100.0) * bar_width as f64) as usize;
    let empty = bar_width - filled;
    let bar = format!(
        "  [{}{}] {:.1}%",
        "#".repeat(filled).green(),
        ".".repeat(empty).dimmed(),
        health_pct
    );
    println!("{}", bar);
    println!();

    Ok(())
}
