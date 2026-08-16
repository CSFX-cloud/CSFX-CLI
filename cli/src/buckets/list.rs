use crate::display::{self, Table};
use crate::http::{auth, base_url, get_json};
use serde::Deserialize;

#[derive(Deserialize)]
struct Bucket {
    id: String,
    name: String,
    exposure: String,
    quota_max_size: Option<i64>,
    status: String,
}

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = auth()?;
    let url = format!("{}/buckets", base_url(&server));

    let pb = display::spinner("fetching buckets...");
    let data = get_json(&client, &url, &token).await;
    pb.finish_and_clear();

    let buckets: Vec<Bucket> = serde_json::from_value(data?)?;

    display::section("Buckets");

    if buckets.is_empty() {
        display::info("no buckets found");
        return Ok(());
    }

    let mut table = Table::new(vec!["ID", "NAME", "EXPOSURE", "QUOTA", "STATUS"]).with_color(
        |col, val| {
            if col == 4 {
                display::status_color(val)
            } else {
                colored::Color::White
            }
        },
    );

    for b in &buckets {
        table.add_row(vec![
            b.id[..8].to_string(),
            b.name.clone(),
            b.exposure.clone(),
            b.quota_max_size
                .map(|s| format!("{}B", s))
                .unwrap_or_else(|| "-".to_string()),
            b.status.clone(),
        ]);
    }

    table.print();
    println!();
    display::info(&format!("{} bucket(s)", buckets.len()));

    Ok(())
}
