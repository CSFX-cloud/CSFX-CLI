mod jobs;
mod target;

use clap::{Args, Subcommand};

#[derive(Args)]
pub struct RunOptions {
    #[arg(long, help = "Export target name, the default target when omitted")]
    target: Option<String>,
    #[arg(long, help = "Wait until the export has finished")]
    wait: bool,
}

#[derive(Subcommand)]
pub enum ExportCommands {
    #[command(about = "Export the platform configuration without tenant data")]
    Config(RunOptions),
    #[command(about = "Export configuration, database and all tenant data")]
    Full(RunOptions),
    #[command(about = "Export the data of one resource group")]
    Group {
        id: String,
        #[command(flatten)]
        options: RunOptions,
    },
    #[command(about = "Export one workload, including the boot disk of a vm")]
    Workload {
        id: String,
        #[command(flatten)]
        options: RunOptions,
    },
    #[command(about = "Export one volume")]
    Volume {
        id: String,
        #[command(flatten)]
        options: RunOptions,
    },
    #[command(about = "Export one object storage bucket")]
    Bucket {
        id: String,
        #[command(flatten)]
        options: RunOptions,
    },
    #[command(about = "List recent exports")]
    List,
    #[command(about = "Show one export")]
    Get { id: String },
    #[command(about = "Restore a configuration, volume, bucket, workload disk or resource group")]
    Restore {
        id: String,
        #[arg(long, help = "Export target the export was written to")]
        target: Option<String>,
        #[arg(long, help = "Agent that receives a restored vm boot disk")]
        agent: Option<String>,
        #[arg(long, help = "Prompt for the passphrase instead of using the target's")]
        prompt_passphrase: bool,
        #[arg(long, help = "Skip the confirmation prompt")]
        yes: bool,
    },
    #[command(about = "Save one decrypted and verified artifact of an export to a file")]
    Download {
        id: String,
        artifact: String,
        #[arg(long, short, help = "File to write")]
        output: String,
        #[arg(long, help = "Export target the export was written to")]
        target: Option<String>,
        #[arg(long, help = "Prompt for the passphrase instead of using the target's")]
        prompt_passphrase: bool,
    },
    #[command(subcommand, about = "Configure where exports are written")]
    Target(TargetCommands),
}

#[derive(Subcommand)]
pub enum TargetCommands {
    #[command(about = "List export targets")]
    List,
    #[command(about = "Add or update a target on a path of the gateway host")]
    AddLocal {
        name: String,
        path: String,
        #[command(flatten)]
        options: TargetOptions,
    },
    #[command(about = "Add or update an S3 compatible target")]
    AddS3 {
        name: String,
        #[arg(long)]
        endpoint: String,
        #[arg(long)]
        bucket: String,
        #[arg(long)]
        region: String,
        #[arg(long, default_value = "")]
        prefix: String,
        #[arg(long)]
        access_key_id: String,
        #[arg(
            long,
            help = "Secret access key, CSFX_EXPORT_S3_SECRET or a prompt when omitted"
        )]
        secret_access_key: Option<String>,
        #[command(flatten)]
        options: TargetOptions,
    },
    #[command(about = "Remove a target, existing exports stay where they are")]
    Remove { name: String },
    #[command(about = "Write, read and delete a probe object on a target")]
    Test { name: String },
}

#[derive(Args)]
pub struct TargetOptions {
    #[arg(long, help = "Encrypt exports with a passphrase that is prompted for")]
    encrypt: bool,
    #[arg(long, help = "Use this target when none is named")]
    default: bool,
}

pub async fn run(cmd: ExportCommands) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        ExportCommands::Config(options) => jobs::create("config", None, options).await,
        ExportCommands::Full(options) => jobs::create("full", None, options).await,
        ExportCommands::Group { id, options } => {
            jobs::create("resource_group", Some(id), options).await
        }
        ExportCommands::Workload { id, options } => {
            jobs::create("workload", Some(id), options).await
        }
        ExportCommands::Volume { id, options } => jobs::create("volume", Some(id), options).await,
        ExportCommands::Bucket { id, options } => jobs::create("bucket", Some(id), options).await,
        ExportCommands::List => jobs::list().await,
        ExportCommands::Get { id } => jobs::get(&id).await,
        ExportCommands::Restore {
            id,
            target,
            agent,
            prompt_passphrase,
            yes,
        } => jobs::restore(&id, target, agent, prompt_passphrase, yes).await,
        ExportCommands::Download {
            id,
            artifact,
            output,
            target,
            prompt_passphrase,
        } => jobs::download(&id, &artifact, &output, target, prompt_passphrase).await,
        ExportCommands::Target(cmd) => target::run(cmd).await,
    }
}
