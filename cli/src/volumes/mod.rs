pub mod attach;
pub mod create;
pub mod delete;
pub mod detach;
pub mod get;
pub mod list;
pub mod snapshots;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum VolumeCommands {
    List,
    Get {
        id: String,
    },
    Create {
        name: String,
        #[arg(long)]
        size: i32,
        #[arg(long)]
        pool: Option<String>,
    },
    Delete {
        id: String,
    },
    Attach {
        id: String,
        #[arg(long)]
        agent: String,
        #[arg(long)]
        workload: Option<String>,
    },
    Detach {
        id: String,
    },
    Snapshots {
        #[arg(long)]
        volume: String,
    },
    SnapshotCreate {
        #[arg(long)]
        volume: String,
        #[arg(long)]
        name: String,
    },
}

pub async fn run(cmd: VolumeCommands) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        VolumeCommands::List => list::run().await,
        VolumeCommands::Get { id } => get::run(&id).await,
        VolumeCommands::Create { name, size, pool } => create::run(name, size, pool).await,
        VolumeCommands::Delete { id } => delete::run(&id).await,
        VolumeCommands::Attach {
            id,
            agent,
            workload,
        } => attach::run(&id, &agent, workload).await,
        VolumeCommands::Detach { id } => detach::run(&id).await,
        VolumeCommands::Snapshots { volume } => snapshots::list(&volume).await,
        VolumeCommands::SnapshotCreate { volume, name } => snapshots::create(&volume, &name).await,
    }
}
