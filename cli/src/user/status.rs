use crate::config::load_config;
use crate::display::{self, kv, kv_colored, section};
use colored::Color;
use reqwest;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct UserInfo {
    user_id: String,
    username: String,
    email: Option<String>,
    two_factor_enabled: bool,
}

pub async fn status() -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config();

    match config {
        Some(cfg) => {
            section("Session");
            kv("Server", &cfg.server);

            match &cfg.token {
                Some(token) => {
                    kv_colored("Status", "authenticated", Color::Green);

                    let pb = display::spinner("fetching user info...");
                    let client = reqwest::Client::new();
                    let url = format!("{}/api/user/me", cfg.server.trim_end_matches('/'));

                    let response = client
                        .get(&url)
                        .header("accept", "application/json")
                        .header("Authorization", format!("Bearer {}", token))
                        .send()
                        .await;
                    pb.finish_and_clear();

                    match response {
                        Ok(resp) if resp.status().is_success() => {
                            let user_info: UserInfo = resp.json().await?;
                            section("User");
                            kv("ID", &user_info.user_id);
                            kv("Username", &user_info.username);
                            if let Some(email) = user_info.email {
                                kv("Email", &email);
                            }
                            kv_colored(
                                "2FA",
                                if user_info.two_factor_enabled {
                                    "enabled"
                                } else {
                                    "disabled"
                                },
                                if user_info.two_factor_enabled {
                                    Color::Green
                                } else {
                                    Color::Yellow
                                },
                            );
                        }
                        Ok(resp) => {
                            display::warn(&format!(
                                "token may be expired ({}), run: csfx login",
                                resp.status()
                            ));
                        }
                        Err(e) => {
                            display::warn(&format!("could not reach server: {}", e));
                        }
                    }
                }
                None => {
                    kv_colored("Status", "not authenticated", Color::Red);
                    display::info("run: csfx login");
                }
            }
        }
        None => {
            display::warn("no configuration found, run: csfx login");
        }
    }

    println!();
    Ok(())
}
