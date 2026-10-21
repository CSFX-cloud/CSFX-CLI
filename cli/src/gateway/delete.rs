use crate::display;
use crate::http::{auth, base_url, delete_req};

pub async fn run(resource_group: &str, id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = auth()?;
    let url = format!(
        "{}/resource-groups/{}/gateways/{}",
        base_url(&server),
        resource_group,
        id
    );

    let pb = display::spinner("deleting gateway...");
    let result = delete_req(&client, &url, &token).await;
    pb.finish_and_clear();

    result?;
    display::success(&format!("gateway {} deleted together with its routes", id));

    Ok(())
}
