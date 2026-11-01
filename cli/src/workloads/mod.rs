pub mod create;
pub mod delete;
pub mod get;
pub mod list;
pub mod metrics;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum WorkloadCommands {
    List,
    Get {
        id: String,
    },
    Create {
        name: String,
        image: String,
        #[arg(long, default_value = "500")]
        cpu: i32,
        #[arg(long, default_value = "536870912")]
        memory: i64,
        #[arg(long, default_value = "10737418240")]
        disk: i64,
    },
    Delete {
        id: String,
    },
    Metrics {
        id: String,
        #[arg(long, short)]
        watch: bool,
    },
}

pub async fn run(cmd: WorkloadCommands) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        WorkloadCommands::List => list::run().await,
        WorkloadCommands::Get { id } => get::run(&id).await,
        WorkloadCommands::Create {
            name,
            image,
            cpu,
            memory,
            disk,
        } => create::run(name, image, cpu, memory, disk).await,
        WorkloadCommands::Delete { id } => delete::run(&id).await,
        WorkloadCommands::Metrics { id, watch } => metrics::run(&id, watch).await,
    }
}
