pub mod record;
pub mod trust;
pub mod zone;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum DnsCommands {
    #[command(about = "Show the platform zone and how to resolve it")]
    Zone,
    #[command(about = "Download the cluster CA and print how to trust it")]
    Trust,
    #[command(subcommand, about = "Manage manual records inside the platform zone")]
    Record(RecordCommands),
}

#[derive(Subcommand)]
pub enum RecordCommands {
    List {
        resource_group: String,
    },
    Add {
        resource_group: String,
        name: String,
        record_type: String,
        value: String,
        #[arg(long)]
        ttl: Option<i32>,
    },
    Delete {
        resource_group: String,
        id: String,
    },
}

pub async fn run(cmd: DnsCommands) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        DnsCommands::Zone => zone::run().await,
        DnsCommands::Trust => trust::run().await,
        DnsCommands::Record(RecordCommands::List { resource_group }) => {
            record::list(&resource_group).await
        }
        DnsCommands::Record(RecordCommands::Add {
            resource_group,
            name,
            record_type,
            value,
            ttl,
        }) => record::add(&resource_group, &name, &record_type, &value, ttl).await,
        DnsCommands::Record(RecordCommands::Delete { resource_group, id }) => {
            record::delete(&resource_group, &id).await
        }
    }
}
