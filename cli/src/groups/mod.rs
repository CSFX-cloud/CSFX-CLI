mod list;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum GroupCommands {
    #[command(about = "List all resource groups")]
    List,
}

pub async fn run(cmd: GroupCommands) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        GroupCommands::List => list::run().await,
    }
}
