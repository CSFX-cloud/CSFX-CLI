use crate::{display, http};

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = http::auth()?;
    let url = format!("{}/organization/roles", http::base_url(&server));

    let pb = display::spinner("fetching roles...");
    let data = http::get_json(&client, &url, &token).await?;
    pb.finish_and_clear();

    let roles = data.as_array().ok_or("unexpected response format")?;

    if roles.is_empty() {
        display::info("no roles found");
        return Ok(());
    }

    let mut table = display::Table::new(vec!["ID", "Name", "Description", "System"]);
    for r in roles {
        table.add_row(vec![
            r["id"]
                .as_str()
                .unwrap_or("-")
                .chars()
                .take(8)
                .collect::<String>()
                + "...",
            r["name"].as_str().unwrap_or("-").to_string(),
            r["description"].as_str().unwrap_or("-").to_string(),
            if r["is_system_role"].as_bool().unwrap_or(false) {
                "yes".to_string()
            } else {
                "no".to_string()
            },
        ]);
    }
    table.print();
    println!();
    Ok(())
}
