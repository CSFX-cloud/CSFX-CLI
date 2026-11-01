use crate::{display, http};

pub async fn run(id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = http::auth()?;
    let url = format!("{}/organization/users/{}", http::base_url(&server), id);

    let pb = display::spinner("fetching user...");
    let u = http::get_json(&client, &url, &token).await?;
    pb.finish_and_clear();

    display::section("User");
    display::kv("ID", u["id"].as_str().unwrap_or("-"));
    display::kv("Username", u["username"].as_str().unwrap_or("-"));
    display::kv("Email", u["email"].as_str().unwrap_or("-"));
    display::kv("Role", u["role_name"].as_str().unwrap_or("-"));
    display::kv(
        "2FA",
        if u["two_factor_enabled"].as_bool().unwrap_or(false) {
            "enabled"
        } else {
            "disabled"
        },
    );
    display::kv(
        "Force PW Change",
        if u["force_password_change"].as_bool().unwrap_or(false) {
            "yes"
        } else {
            "no"
        },
    );
    display::kv("Joined", u["joined_at"].as_str().unwrap_or("-"));
    println!();
    Ok(())
}
