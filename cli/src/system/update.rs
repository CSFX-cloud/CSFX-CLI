use crate::display;
use crate::http::{auth, base_url, get_json, post_json};
use colored::Colorize;
use serde::Deserialize;
use serde_json::json;
use std::io::Write;
use std::time::{Duration, Instant};
use tokio::time::sleep;

#[derive(Debug, Deserialize)]
struct UpdateStatus {
    current_version: String,
    desired_version: Option<String>,
    available_flake_rev: Option<String>,
    desired_flake_rev: Option<String>,
    build_status: Option<String>,
    last_result: Option<String>,
    paused: bool,
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Phase {
    Pending,
    Active,
    Done,
    Failed,
}

struct Step {
    label: &'static str,
    phase: Phase,
    detail: Option<String>,
}

pub async fn run(version: String) -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = auth()?;
    let url = format!("{}/system/update", base_url(&server));

    let pb = display::spinner(&format!("scheduling update to {}", version));
    let resp = post_json(&client, &url, &token, &json!({ "version": version })).await?;
    pb.finish_and_clear();

    let status = resp["status"].as_str().unwrap_or("-");
    if status != "update_scheduled" {
        display::error(&format!("unexpected status: {}", status));
        return Ok(());
    }

    display::success(&format!("update scheduled: {}", version));
    println!();

    watch_loop(&client, &server, &token, Some(version)).await
}

pub async fn run_status() -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = auth()?;
    watch_loop(&client, &server, &token, None).await
}

pub async fn run_pause() -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = auth()?;
    let url = format!("{}/system/update/pause", base_url(&server));

    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    if resp.status().is_success() {
        display::success("updates paused");
    } else {
        display::error(&format!("failed to pause updates: {}", resp.status()));
    }

    Ok(())
}

pub async fn run_resume() -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = auth()?;
    let url = format!("{}/system/update/resume", base_url(&server));

    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    if resp.status().is_success() {
        display::success("updates resumed");
    } else {
        display::error(&format!("failed to resume updates: {}", resp.status()));
    }

    Ok(())
}

async fn fetch_status(
    client: &reqwest::Client,
    server: &str,
    token: &str,
) -> Result<UpdateStatus, Box<dyn std::error::Error>> {
    let url = format!("{}/system/update/status", base_url(server));
    let val = get_json(client, &url, token).await?;
    Ok(serde_json::from_value(val)?)
}

fn build_steps(status: &UpdateStatus, target_version: Option<&str>) -> Vec<Step> {
    let last = status.last_result.as_deref();
    let build = status.build_status.as_deref();
    let globally_failed = last == Some("failed");

    let phase1 = if status.desired_version.is_some() {
        if status.available_flake_rev.is_some() {
            Phase::Done
        } else if globally_failed || build == Some("failed") {
            Phase::Failed
        } else {
            Phase::Active
        }
    } else {
        Phase::Pending
    };

    let phase2 = match phase1 {
        Phase::Done => {
            if status.available_flake_rev == status.desired_flake_rev {
                Phase::Done
            } else if globally_failed {
                Phase::Failed
            } else {
                Phase::Active
            }
        }
        Phase::Active => Phase::Pending,
        _ => Phase::Pending,
    };

    let phase3 = match phase2 {
        Phase::Done => match build {
            Some("building") => Phase::Active,
            Some("ready") | Some("failed") if last == Some("success") => Phase::Done,
            Some("failed") => Phase::Failed,
            _ => Phase::Pending,
        },
        _ => Phase::Pending,
    };

    let phase4 = match phase3 {
        Phase::Done => {
            if last == Some("success") {
                Phase::Done
            } else {
                Phase::Pending
            }
        }
        Phase::Active => Phase::Pending,
        Phase::Failed => Phase::Pending,
        Phase::Pending => Phase::Pending,
    };

    let rev_short = status
        .available_flake_rev
        .as_deref()
        .map(|r| format!("sha:{}", &r[..8.min(r.len())]));

    let target = target_version
        .or(status.desired_version.as_deref())
        .unwrap_or("?");

    vec![
        Step {
            label: "Resolving version to flake rev",
            phase: phase1,
            detail: rev_short,
        },
        Step {
            label: "Mirroring repository",
            phase: phase2,
            detail: None,
        },
        Step {
            label: "Building NixOS closure",
            phase: phase3,
            detail: None,
        },
        Step {
            label: "Applying (nixos-rebuild switch)",
            phase: phase4,
            detail: status
                .last_result
                .as_deref()
                .filter(|_| phase4 == Phase::Done)
                .map(|_| format!("-> {}", target)),
        },
    ]
}

fn render(
    steps: &[Step],
    status: &UpdateStatus,
    elapsed: Duration,
    tick: u8,
    target: Option<&str>,
) {
    let spinner_frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let frame = spinner_frames[(tick as usize) % spinner_frames.len()];

    print!("\x1B[2J\x1B[H");

    let target_label = target.or(status.desired_version.as_deref()).unwrap_or("?");

    println!("  {} {}", "Update:".bold(), target_label.yellow().bold());

    if status.paused {
        println!("  {}", "updates paused".yellow());
    }

    let secs = elapsed.as_secs();
    println!(
        "  {} {:02}:{:02}",
        "Elapsed:".dimmed(),
        secs / 60,
        secs % 60
    );
    println!();

    for (i, step) in steps.iter().enumerate() {
        let (icon, label_str) = match step.phase {
            Phase::Done => ("✔".green().bold().to_string(), step.label.to_string()),
            Phase::Failed => ("✖".red().bold().to_string(), step.label.red().to_string()),
            Phase::Active => (
                frame.cyan().bold().to_string(),
                step.label.bold().to_string(),
            ),
            Phase::Pending => ("─".dimmed().to_string(), step.label.dimmed().to_string()),
        };

        let detail = step
            .detail
            .as_deref()
            .map(|d| format!("  {}", d.dimmed()))
            .unwrap_or_default();

        println!("  [{}/4] {} {}  {}", i + 1, icon, label_str, detail);
    }

    println!();

    let current = status.current_version.as_str();
    display::kv("current", current);

    if let Some(ref desired) = status.desired_version {
        display::kv("target", desired.as_str());
    }

    if let Some(ref result) = status.last_result {
        let color = match result.as_str() {
            "success" => colored::Color::Green,
            "failed" => colored::Color::Red,
            _ => colored::Color::Yellow,
        };
        display::kv_colored("last result", result, color);
    }

    println!();
    println!(
        "  {}",
        "Ctrl+C to exit (update continues in background)".dimmed()
    );

    std::io::stdout().flush().ok();
}

async fn watch_loop(
    client: &reqwest::Client,
    server: &str,
    token: &str,
    target_version: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();
    let poll_interval = Duration::from_secs(5);
    let render_interval = Duration::from_millis(100);

    let mut status = fetch_status(client, server, token).await?;
    let mut last_poll = Instant::now();
    let mut tick: u8 = 0;

    loop {
        if last_poll.elapsed() >= poll_interval {
            match fetch_status(client, server, token).await {
                Ok(s) => {
                    status = s;
                    last_poll = Instant::now();
                }
                Err(e) => {
                    display::error(&format!("fetch failed: {}", e));
                    last_poll = Instant::now();
                }
            }
        }

        let steps = build_steps(&status, target_version.as_deref());
        render(
            &steps,
            &status,
            start.elapsed(),
            tick,
            target_version.as_deref(),
        );

        let done = steps
            .iter()
            .enumerate()
            .any(|(i, s)| i == 3 && s.phase == Phase::Done);
        let failed = steps.iter().any(|s| s.phase == Phase::Failed);

        if done || failed {
            break;
        }

        sleep(render_interval).await;
        tick = tick.wrapping_add(1);
    }

    Ok(())
}
