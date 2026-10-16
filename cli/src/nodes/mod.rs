pub mod agent_metrics;
pub mod get;
pub mod lifecycle;
pub mod list;
pub mod metrics;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum NodeCommands {
    List,
    Get { id: String },
    JoinToken {
        #[arg(long, default_value_t = 1)]
        ttl: i64,
    },
    Cordon { id: String },
    Uncordon { id: String },
    Drain { id: String },
    Remove { id: String },
    Metrics {
        #[arg(long, short)]
        watch: bool,
    },
    AgentMetrics {
        id: String,
        #[arg(long, short)]
        watch: bool,
    },
}

pub async fn run(cmd: NodeCommands) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        NodeCommands::List => list::run().await,
        NodeCommands::Get { id } => get::run(&id).await,
        NodeCommands::JoinToken { ttl } => lifecycle::join_token(ttl).await,
        NodeCommands::Cordon { id } => lifecycle::cordon(&id).await,
        NodeCommands::Uncordon { id } => lifecycle::uncordon(&id).await,
        NodeCommands::Drain { id } => lifecycle::drain(&id).await,
        NodeCommands::Remove { id } => lifecycle::remove(&id).await,
        NodeCommands::Metrics { watch } => metrics::run(watch).await,
        NodeCommands::AgentMetrics { id, watch } => agent_metrics::run(&id, watch).await,
    }
}
