pub mod create;
pub mod delete;
pub mod list;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum GatewayCommands {
    List {
        resource_group: String,
    },
    Create {
        resource_group: String,
        name: String,
        #[arg(long, default_value_t = 80)]
        http_port: u16,
        #[arg(long, default_value_t = 443)]
        https_port: u16,
    },
    Delete {
        resource_group: String,
        id: String,
    },
}

pub async fn run(cmd: GatewayCommands) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        GatewayCommands::List { resource_group } => list::run(&resource_group).await,
        GatewayCommands::Create {
            resource_group,
            name,
            http_port,
            https_port,
        } => create::run(&resource_group, &name, http_port, https_port).await,
        GatewayCommands::Delete { resource_group, id } => delete::run(&resource_group, &id).await,
    }
}
