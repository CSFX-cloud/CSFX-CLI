pub mod access;
pub mod connect;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum SshCommands {
    #[command(about = "Open an SSH session to a cluster node")]
    Connect {
        #[arg(help = "Node name or UUID")]
        node: String,
        #[arg(long, short, help = "SSH port", default_value = "22")]
        port: u16,
    },
    #[command(about = "Allow SSH access to all cluster nodes")]
    Enable {
        #[arg(long, help = "Hours until access closes again (default 8)")]
        hours: Option<i64>,
        #[arg(long, conflicts_with = "hours", help = "Keep access open until disabled")]
        unlimited: bool,
    },
    #[command(about = "Close SSH access on all cluster nodes")]
    Disable,
    #[command(about = "Show whether SSH access is open")]
    Status,
}

pub async fn run(cmd: SshCommands) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        SshCommands::Connect { node, port } => connect::run(node, port).await,
        SshCommands::Enable { hours, unlimited } => access::enable(hours, unlimited).await,
        SshCommands::Disable => access::disable().await,
        SshCommands::Status => access::status().await,
    }
}
