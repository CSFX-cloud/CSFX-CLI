use super::{TargetCommands, TargetOptions};
use crate::display::{self, Table};
use crate::http::{auth, base_url, get_json, request_json};
use dialoguer::Password;
use reqwest::Method;
use serde_json::{json, Value};

const SECRET_VARIABLE: &str = "CSFX_EXPORT_S3_SECRET";

fn endpoint(server: &str, path: &str) -> String {
    format!("{}/admin/export-targets{}", base_url(server), path)
}

fn prompt_passphrase() -> Result<String, Box<dyn std::error::Error>> {
    display::warn("exports cannot be decrypted without this passphrase, store it safely");
    let passphrase = Password::new()
        .with_prompt("passphrase (min 12 characters)")
        .with_confirmation("repeat passphrase", "passphrases differ")
        .interact()?;
    Ok(passphrase)
}

fn with_options(
    mut body: Value,
    options: &TargetOptions,
) -> Result<Value, Box<dyn std::error::Error>> {
    let passphrase = if options.encrypt {
        Some(prompt_passphrase()?)
    } else {
        None
    };
    body["passphrase"] = json!(passphrase);
    body["default"] = json!(options.default);
    Ok(body)
}

async fn put(name: &str, body: Value) -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = auth()?;
    let url = endpoint(&server, &format!("/{name}"));
    request_json(&client, Method::PUT, &url, &token, Some(&body)).await?;
    display::success(&format!("export target saved: {name}"));
    Ok(())
}

async fn list() -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = auth()?;
    let targets = get_json(&client, &endpoint(&server, ""), &token).await?;
    let rows = targets.as_array().ok_or("unexpected response")?;
    display::section("Export targets");
    if rows.is_empty() {
        display::info("no targets, add one with: csfx export target add-local");
        return Ok(());
    }
    let mut table = Table::new(vec!["NAME", "KIND", "LOCATION", "ENCRYPTED", "DEFAULT"]);
    for row in rows {
        table.add_row(vec![
            row["name"].as_str().unwrap_or("-").to_string(),
            row["kind"].as_str().unwrap_or("-").to_string(),
            row["location"].as_str().unwrap_or("-").to_string(),
            row["encrypted"].as_bool().unwrap_or(false).to_string(),
            row["default"].as_bool().unwrap_or(false).to_string(),
        ]);
    }
    table.print();
    Ok(())
}

fn secret(provided: Option<String>) -> Result<String, Box<dyn std::error::Error>> {
    if let Some(secret) = provided.or_else(|| std::env::var(SECRET_VARIABLE).ok()) {
        return Ok(secret);
    }
    Ok(Password::new()
        .with_prompt("secret access key")
        .interact()?)
}

pub async fn run(cmd: TargetCommands) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        TargetCommands::List => list().await,
        TargetCommands::AddLocal {
            name,
            path,
            options,
        } => {
            put(
                &name,
                with_options(json!({ "kind": "local", "path": path }), &options)?,
            )
            .await
        }
        TargetCommands::AddS3 {
            name,
            endpoint,
            bucket,
            region,
            prefix,
            access_key_id,
            secret_access_key,
            options,
        } => {
            let body = json!({
                "kind": "s3",
                "endpoint": endpoint,
                "bucket": bucket,
                "region": region,
                "prefix": prefix,
                "access_key_id": access_key_id,
                "secret_access_key": secret(secret_access_key)?,
            });
            put(&name, with_options(body, &options)?).await
        }
        TargetCommands::Remove { name } => {
            let (client, server, token) = auth()?;
            let url = endpoint(&server, &format!("/{name}"));
            request_json(&client, Method::DELETE, &url, &token, None).await?;
            display::success(&format!("export target removed: {name}"));
            Ok(())
        }
        TargetCommands::Test { name } => {
            let (client, server, token) = auth()?;
            let url = endpoint(&server, &format!("/{name}/test"));
            request_json(&client, Method::POST, &url, &token, None).await?;
            display::success(&format!("export target works: {name}"));
            Ok(())
        }
    }
}
