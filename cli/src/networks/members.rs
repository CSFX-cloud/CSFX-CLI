use crate::display::{self, section, Table};
use crate::http::{auth, base_url, delete_req, get_json, post_json};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
struct Member {
    id: String,
    workload_id: String,
    allocated_ip: String,
    created_at: String,
}

pub async fn list(network_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = auth()?;
    let url = format!("{}/networks/{}/members", base_url(&server), network_id);

    let pb = display::spinner("fetching members...");
    let data = get_json(&client, &url, &token).await;
    pb.finish_and_clear();

    let data = data?;
    let members: Vec<Member> = serde_json::from_value(data)?;

    section("Networks  /  Members");

    if members.is_empty() {
        display::info("no members");
        return Ok(());
    }

    let mut table =
        Table::new(vec!["ID", "WORKLOAD", "IP", "JOINED"]).with_color(|_, _| colored::Color::White);

    for m in &members {
        table.add_row(vec![
            m.id[..8].to_string(),
            m.workload_id[..8].to_string(),
            m.allocated_ip.clone(),
            m.created_at[..16.min(m.created_at.len())].to_string(),
        ]);
    }

    table.print();
    println!();
    display::info(&format!("{} member(s)", members.len()));

    Ok(())
}

pub async fn add(network_id: &str, workload_id: String) -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = auth()?;
    let url = format!("{}/networks/{}/members", base_url(&server), network_id);

    let body = json!({ "workload_id": workload_id });

    let pb = display::spinner("adding member...");
    let data = post_json(&client, &url, &token, &body).await;
    pb.finish_and_clear();

    let data = data?;
    let member: Member = serde_json::from_value(data)?;

    section("Networks  /  Member Add");
    display::success(&format!(
        "workload joined network, allocated ip={}",
        member.allocated_ip
    ));

    Ok(())
}

pub async fn remove(network_id: &str, workload_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = auth()?;
    let url = format!(
        "{}/networks/{}/members/{}",
        base_url(&server),
        network_id,
        workload_id
    );

    let pb = display::spinner("removing member...");
    let result = delete_req(&client, &url, &token).await;
    pb.finish_and_clear();

    result?;

    section("Networks  /  Member Remove");
    display::success("workload removed from network");

    Ok(())
}
