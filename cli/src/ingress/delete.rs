use crate::display;
use crate::http::{auth, base_url, delete_req};

pub async fn run(resource_group: &str, id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (client, server, token) = auth()?;
    let url = format!(
        "{}/resource-groups/{}/ingress/{}",
        base_url(&server),
        resource_group,
        id
    );

    let pb = display::spinner("deleting ingress route...");
    let result = delete_req(&client, &url, &token).await;
    pb.finish_and_clear();

    result?;
    display::success(&format!("ingress route {} deleted", id));

    Ok(())
}
