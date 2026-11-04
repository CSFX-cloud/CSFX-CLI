mod privileged;
mod resolve;
mod resolver;
mod tunnel;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum ConnectCommands {
    #[command(about = "Join the private WireGuard network of a resource group")]
    Rg {
        name: String,
        #[arg(long, help = "Request a new WireGuard configuration instead of reusing the stored one")]
        refresh: bool,
        #[arg(long, help = "Leave the network again")]
        down: bool,
    },
}

pub async fn run(cmd: ConnectCommands) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        ConnectCommands::Rg {
            name,
            refresh,
            down,
        } => {
            let group = resolve::find_group(&name).await?;
            if down {
                tunnel::down(&group).await
            } else {
                tunnel::up(&group, refresh).await
            }
        }
    }
}
