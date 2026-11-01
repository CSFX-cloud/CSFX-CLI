use crate::{display, http};

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = http::auth()?;
    let url = format!("{}/organization/users", http::base_url(&server));

    let pb = display::spinner("fetching users...");
    let data = http::get_json(&client, &url, &token).await?;
    pb.finish_and_clear();

    let users = data.as_array().ok_or("unexpected response format")?;

    if users.is_empty() {
        display::info("no users found");
        return Ok(());
    }

    let mut table = display::Table::new(vec!["ID", "Username", "Email", "Role", "2FA", "Joined"]);
    for u in users {
        table.add_row(vec![
            u["id"]
                .as_str()
                .unwrap_or("-")
                .chars()
                .take(8)
                .collect::<String>()
                + "...",
            u["username"].as_str().unwrap_or("-").to_string(),
            u["email"].as_str().unwrap_or("-").to_string(),
            u["role_name"].as_str().unwrap_or("-").to_string(),
            if u["two_factor_enabled"].as_bool().unwrap_or(false) {
                "on".to_string()
            } else {
                "off".to_string()
            },
            u["joined_at"]
                .as_str()
                .unwrap_or("-")
                .chars()
                .take(10)
                .collect(),
        ]);
    }
    table.print();
    println!();
    Ok(())
}
