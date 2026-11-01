pub mod create;
pub mod delete;
pub mod get;
pub mod list;
pub mod members;
pub mod policies;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum NetworkCommands {
    List,
    Get {
        id: String,
    },
    Create {
        name: String,
        cidr: String,
        #[arg(long, default_value = "wireguard")]
        overlay: String,
    },
    Delete {
        id: String,
    },
    Policies {
        #[arg(long)]
        network: String,
    },
    PolicyCreate {
        #[arg(long)]
        network: String,
        #[arg(long)]
        direction: String,
        #[arg(long)]
        action: String,
        #[arg(long, default_value = "100")]
        priority: i32,
        #[arg(long)]
        source: Option<String>,
        #[arg(long)]
        destination: Option<String>,
        #[arg(long)]
        port: Option<i32>,
        #[arg(long)]
        protocol: Option<String>,
    },
    Members {
        #[arg(long)]
        network: String,
    },
    MemberAdd {
        #[arg(long)]
        network: String,
        #[arg(long)]
        workload: String,
    },
    MemberRemove {
        #[arg(long)]
        network: String,
        #[arg(long)]
        workload: String,
    },
}

pub async fn run(cmd: NetworkCommands) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        NetworkCommands::List => list::run().await,
        NetworkCommands::Get { id } => get::run(&id).await,
        NetworkCommands::Create {
            name,
            cidr,
            overlay,
        } => create::run(name, cidr, overlay).await,
        NetworkCommands::Delete { id } => delete::run(&id).await,
        NetworkCommands::Policies { network } => policies::list(&network).await,
        NetworkCommands::PolicyCreate {
            network,
            direction,
            action,
            priority,
            source,
            destination,
            port,
            protocol,
        } => {
            policies::create(
                &network,
                direction,
                action,
                priority,
                source,
                destination,
                port,
                protocol,
            )
            .await
        }
        NetworkCommands::Members { network } => members::list(&network).await,
        NetworkCommands::MemberAdd { network, workload } => members::add(&network, workload).await,
        NetworkCommands::MemberRemove { network, workload } => {
            members::remove(&network, &workload).await
        }
    }
}
