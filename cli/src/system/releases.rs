use crate::display;
use colored::Colorize;
use serde::Deserialize;

const RELEASES_API: &str = "https://api.github.com/repos/CSFX-cloud/CSF-Core/releases";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    prerelease: bool,
    html_url: String,
    name: Option<String>,
}

pub async fn run(include_pre: bool) -> Result<(), Box<dyn std::error::Error>> {
    let pb = display::spinner("fetching releases");
    let releases = fetch_releases().await?;
    pb.finish_and_clear();

    let filtered: Vec<&Release> = releases
        .iter()
        .filter(|r| include_pre || !r.prerelease)
        .take(10)
        .collect();

    if filtered.is_empty() {
        display::info("no releases found");
        return Ok(());
    }

    display::section("Available Releases");
    println!(
        "  {:<20} {:<12} {}",
        "VERSION".bold(),
        "TYPE".bold(),
        "CURRENT".bold()
    );
    println!("  {}", "─".repeat(50).dimmed());

    for release in &filtered {
        let version = release.tag_name.trim_start_matches('v');
        let is_current = version == CURRENT_VERSION;
        let kind = if release.prerelease {
            "pre-release"
        } else {
            "stable"
        };

        let version_str = if is_current {
            format!("{} (running)", version).green().bold().to_string()
        } else if is_newer(version, CURRENT_VERSION) {
            version.yellow().bold().to_string()
        } else {
            version.dimmed().to_string()
        };

        let kind_str = if release.prerelease {
            kind.yellow().to_string()
        } else {
            kind.green().to_string()
        };

        println!(
            "  {:<20} {:<12} {}",
            version_str,
            kind_str,
            if is_current { "*" } else { "" }
        );
    }

    println!();

    let latest = filtered
        .iter()
        .find(|r| !r.prerelease)
        .or_else(|| filtered.first());

    if let Some(latest) = latest {
        let latest_version = latest.tag_name.trim_start_matches('v');
        if is_newer(latest_version, CURRENT_VERSION) {
            display::info(&format!(
                "update available: {} -> {}",
                CURRENT_VERSION, latest_version
            ));
            display::info(&format!("run: csfx system update {}", latest_version));
        } else {
            display::success(&format!("up to date ({})", CURRENT_VERSION));
        }
    }

    Ok(())
}

async fn fetch_releases() -> Result<Vec<Release>, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let resp = client
        .get(RELEASES_API)
        .header("User-Agent", "csfx-cli")
        .header("Accept", "application/vnd.github+json")
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(format!("github api error: {}", resp.status()).into());
    }

    Ok(resp.json::<Vec<Release>>().await?)
}

fn is_newer(candidate: &str, current: &str) -> bool {
    parse_semver(candidate)
        .zip(parse_semver(current))
        .map(|(c, cur)| c > cur)
        .unwrap_or(false)
}

fn parse_semver(v: &str) -> Option<(u32, u32, u32, Option<String>)> {
    let v = v.trim_start_matches('v');
    let (base, pre) = match v.split_once('-') {
        Some((b, p)) => (b, Some(p.to_string())),
        None => (v, None),
    };
    let parts: Vec<&str> = base.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    Some((
        parts[0].parse().ok()?,
        parts[1].parse().ok()?,
        parts[2].parse().ok()?,
        pre,
    ))
}
