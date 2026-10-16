pub mod add;
pub mod delete;
pub mod list;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum IngressCommands {
    List {
        resource_group: String,
    },
    Add {
        resource_group: String,
        hostname: String,
        #[arg(long)]
        workload: String,
        #[arg(long)]
        port: u16,
        #[arg(long)]
        no_tls: bool,
    },
    Delete {
        resource_group: String,
        id: String,
    },
}

pub async fn run(cmd: IngressCommands) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        IngressCommands::List { resource_group } => list::run(&resource_group).await,
        IngressCommands::Add {
            resource_group,
            hostname,
            workload,
            port,
            no_tls,
        } => add::run(&resource_group, &hostname, &workload, port, !no_tls).await,
        IngressCommands::Delete { resource_group, id } => {
            delete::run(&resource_group, &id).await
        }
    }
}
