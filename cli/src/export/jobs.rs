use super::RunOptions;
use crate::display::{self, kv, Table};
use crate::http::{auth, base_url, get_json, post_json, request_json};
use dialoguer::{Confirm, Password};
use reqwest::{Client, Method};
use serde_json::{json, Value};
use std::time::Duration;

const POLL_INTERVAL: Duration = Duration::from_secs(1);

fn text<'a>(value: &'a Value, field: &str) -> &'a str {
    value[field].as_str().unwrap_or("-")
}

pub async fn create(
    kind: &str,
    id: Option<String>,
    options: RunOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = auth()?;
    let url = format!("{}/exports", base_url(&server));
    let body = json!({ "scope": { "kind": kind, "id": id }, "target": options.target });
    let created = post_json(&client, &url, &token, &body).await?;
    let export_id = created["id"]
        .as_str()
        .ok_or("response is missing the export id")?;
    display::success(&format!("export started: {export_id}"));
    if options.wait {
        wait_for(&client, &format!("{url}/{export_id}"), &token).await?;
    }
    Ok(())
}

async fn wait_for(
    client: &Client,
    url: &str,
    token: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let spinner = display::spinner("exporting...");
    loop {
        let export = get_json(client, url, token).await?;
        match export["status"].as_str() {
            Some("running") => tokio::time::sleep(POLL_INTERVAL).await,
            Some("completed") => {
                spinner.finish_and_clear();
                display::success("export completed");
                return Ok(());
            }
            _ => {
                spinner.finish_and_clear();
                return Err(format!("export failed: {}", text(&export, "error")).into());
            }
        }
    }
}

pub async fn list() -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = auth()?;
    let exports = get_json(&client, &format!("{}/exports", base_url(&server)), &token).await?;
    let rows = exports.as_array().ok_or("unexpected response")?;
    display::section("Exports");
    if rows.is_empty() {
        display::info("no exports found, create one with: csfx export config");
        return Ok(());
    }
    let mut table = Table::new(vec!["ID", "SCOPE", "STATUS", "CREATED"]);
    for row in rows {
        table.add_row(vec![
            text(row, "id").to_string(),
            text(row, "scope_kind").to_string(),
            text(row, "status").to_string(),
            text(row, "created_at").to_string(),
        ]);
    }
    table.print();
    Ok(())
}

pub async fn get(id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = auth()?;
    let url = format!("{}/exports/{}", base_url(&server), id);
    let export = get_json(&client, &url, &token).await?;
    display::section("Export");
    kv("ID", text(&export, "id"));
    kv("Scope", text(&export, "scope_kind"));
    kv("Status", text(&export, "status"));
    kv("Created", text(&export, "created_at"));
    kv("Finished", text(&export, "finished_at"));
    let manifest = &export["manifest"];
    if manifest.is_object() {
        kv("Version", text(manifest, "csfx_version"));
        kv("Consistency", text(manifest, "consistency"));
        kv("Encrypted", &manifest["encryption"].is_object().to_string());
        for artifact in manifest["artifacts"].as_array().into_iter().flatten() {
            let size = artifact["bytes"].as_u64().unwrap_or(0);
            kv(
                "Artifact",
                &format!("{} ({size} bytes)", text(artifact, "name")),
            );
        }
    }
    if export["error"].is_string() {
        kv("Error", text(&export, "error"));
    }
    Ok(())
}

fn confirmed(skip: bool) -> Result<bool, Box<dyn std::error::Error>> {
    if skip {
        return Ok(true);
    }
    let prompt = "restoring creates resources or overwrites matching configuration rows, continue?";
    Ok(Confirm::new()
        .with_prompt(prompt)
        .default(false)
        .interact()?)
}

pub async fn restore(
    id: &str,
    target: Option<String>,
    agent: Option<String>,
    prompt_passphrase: bool,
    yes: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if !confirmed(yes)? {
        display::info("restore cancelled");
        return Ok(());
    }
    let passphrase = if prompt_passphrase {
        Some(
            Password::new()
                .with_prompt("export passphrase")
                .interact()?,
        )
    } else {
        None
    };
    let (client, server, token) = auth()?;
    let url = format!("{}/exports/{}/restore", base_url(&server), id);
    let body = json!({ "target": target, "agent_id": agent, "passphrase": passphrase });
    let outcome = request_json(&client, Method::POST, &url, &token, Some(&body)).await?;
    display::success("restore completed");
    if outcome.as_object().is_some_and(|fields| !fields.is_empty()) {
        println!("{}", serde_json::to_string_pretty(&outcome)?);
    }
    Ok(())
}

pub async fn download(
    id: &str,
    artifact: &str,
    output: &str,
    target: Option<String>,
    prompt_passphrase: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = auth()?;
    let mut url = format!(
        "{}/exports/{}/artifacts/{}",
        base_url(&server),
        id,
        artifact
    );
    if let Some(target) = target {
        url.push_str(&format!("?target={target}"));
    }
    let mut request = client.get(&url).bearer_auth(&token);
    if prompt_passphrase {
        let passphrase = Password::new()
            .with_prompt("export passphrase")
            .interact()?;
        request = request.header("X-Export-Passphrase", passphrase);
    }
    let mut response = request.send().await?;
    if !response.status().is_success() {
        let status = response.status();
        return Err(format!("download failed: {status} {}", response.text().await?).into());
    }
    let mut file = tokio::fs::File::create(output).await?;
    while let Some(chunk) = response.chunk().await? {
        tokio::io::AsyncWriteExt::write_all(&mut file, &chunk).await?;
    }
    display::success(&format!("saved {output}"));
    Ok(())
}
