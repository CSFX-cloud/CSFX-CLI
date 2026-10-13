pub mod cluster;
pub mod create;
pub mod delete;
pub mod get;
pub mod keys;
pub mod list;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum BucketCommands {
    List,
    Get {
        id: String,
    },
    Create {
        name: String,
        #[arg(long)]
        resource_group: Option<String>,
        #[arg(long)]
        exposure: Option<String>,
        #[arg(long)]
        quota_size: Option<i64>,
        #[arg(long)]
        quota_objects: Option<i64>,
    },
    Delete {
        id: String,
    },
    Cluster,
    #[command(subcommand)]
    Keys(BucketKeyCommands),
}

#[derive(Subcommand)]
pub enum BucketKeyCommands {
    List {
        #[arg(long)]
        bucket: String,
    },
    Create {
        #[arg(long)]
        bucket: String,
        name: String,
        #[arg(long)]
        permissions: Option<String>,
    },
    Rotate {
        #[arg(long)]
        bucket: String,
        id: String,
    },
    Delete {
        #[arg(long)]
        bucket: String,
        id: String,
    },
}

pub async fn run(cmd: BucketCommands) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        BucketCommands::List => list::run().await,
        BucketCommands::Get { id } => get::run(&id).await,
        BucketCommands::Create {
            name,
            resource_group,
            exposure,
            quota_size,
            quota_objects,
        } => create::run(name, resource_group, exposure, quota_size, quota_objects).await,
        BucketCommands::Delete { id } => delete::run(&id).await,
        BucketCommands::Cluster => cluster::run().await,
        BucketCommands::Keys(key_cmd) => run_keys(key_cmd).await,
    }
}

async fn run_keys(cmd: BucketKeyCommands) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        BucketKeyCommands::List { bucket } => keys::list(&bucket).await,
        BucketKeyCommands::Create {
            bucket,
            name,
            permissions,
        } => keys::create(&bucket, name, permissions).await,
        BucketKeyCommands::Rotate { bucket, id } => keys::rotate(&bucket, &id).await,
        BucketKeyCommands::Delete { bucket, id } => keys::delete(&bucket, &id).await,
    }
}
