pub mod agents;
pub mod bootstrap;
pub mod deregister;
pub mod stats;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum RegistryCommands {
    Agents,
    #[command(name = "agents-get")]
    AgentsGet {
        id: String,
    },
    Deregister {
        id: String,
    },
    Stats,
    #[command(name = "bootstrap-create")]
    BootstrapCreate {
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        ttl: Option<i64>,
        #[arg(long, name = "max-uses")]
        max_uses: Option<i32>,
    },
    #[command(name = "bootstrap-list")]
    BootstrapList,
    #[command(name = "bootstrap-revoke")]
    BootstrapRevoke {
        id: String,
    },
}

pub async fn run(cmd: RegistryCommands) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        RegistryCommands::Agents => agents::list().await,
        RegistryCommands::AgentsGet { id } => agents::get(&id).await,
        RegistryCommands::Deregister { id } => deregister::agent(&id).await,
        RegistryCommands::Stats => stats::run().await,
        RegistryCommands::BootstrapCreate {
            description,
            ttl,
            max_uses,
        } => bootstrap::create(description, ttl, max_uses).await,
        RegistryCommands::BootstrapList => bootstrap::list().await,
        RegistryCommands::BootstrapRevoke { id } => bootstrap::revoke(&id).await,
    }
}
