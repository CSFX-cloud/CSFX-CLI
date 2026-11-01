mod api;
mod audit;
mod grants;
mod secrets;
mod vaults;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum VaultCommands {
    List {
        resource_group: String,
    },
    Create {
        resource_group: String,
        name: String,
    },
    Get {
        resource_group: String,
        vault: String,
    },
    Delete {
        resource_group: String,
        vault: String,
    },
    Seal {
        resource_group: String,
        vault: String,
    },
    Unseal {
        resource_group: String,
        vault: String,
    },
    Audit {
        resource_group: String,
        vault: String,
    },
    #[command(subcommand)]
    Secrets(SecretCommands),
    #[command(subcommand)]
    Grants(GrantCommands),
}

#[derive(Subcommand)]
pub enum SecretCommands {
    List {
        resource_group: String,
        vault: String,
    },
    Set {
        resource_group: String,
        vault: String,
        name: String,
        #[arg(long, help = "Secret value, prompted without echo when omitted")]
        value: Option<String>,
        #[arg(long, help = "Expiry date, e.g. 2027-01-31")]
        expires: Option<String>,
    },
    Delete {
        resource_group: String,
        vault: String,
        name: String,
    },
    Restore {
        resource_group: String,
        vault: String,
        name: String,
        #[arg(long)]
        version: u64,
    },
    Reveal {
        resource_group: String,
        vault: String,
        name: String,
        #[arg(long, help = "Why the value is needed, written to the audit log")]
        reason: String,
        #[arg(long, help = "Print only the value")]
        raw: bool,
    },
}

#[derive(Subcommand)]
pub enum GrantCommands {
    List {
        resource_group: String,
        vault: String,
    },
    Add {
        resource_group: String,
        vault: String,
        #[arg(help = "Resource name for read, user name for reveal and manage")]
        who: String,
        #[arg(help = "read, reveal or manage")]
        can: String,
        #[arg(
            long,
            help = "Limit read access to one secret, all secrets when omitted"
        )]
        on: Option<String>,
        #[arg(long, help = "Every reveal needs approval")]
        approval: bool,
    },
    Remove {
        resource_group: String,
        vault: String,
        id: String,
    },
}

pub async fn run(cmd: VaultCommands) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        VaultCommands::List { resource_group } => vaults::list(&resource_group).await,
        VaultCommands::Create {
            resource_group,
            name,
        } => vaults::create(&resource_group, &name).await,
        VaultCommands::Get {
            resource_group,
            vault,
        } => vaults::get(&resource_group, &vault).await,
        VaultCommands::Delete {
            resource_group,
            vault,
        } => vaults::delete(&resource_group, &vault).await,
        VaultCommands::Seal {
            resource_group,
            vault,
        } => vaults::set_sealed(&resource_group, &vault, true).await,
        VaultCommands::Unseal {
            resource_group,
            vault,
        } => vaults::set_sealed(&resource_group, &vault, false).await,
        VaultCommands::Audit {
            resource_group,
            vault,
        } => audit::run(&resource_group, &vault).await,
        VaultCommands::Secrets(cmd) => run_secrets(cmd).await,
        VaultCommands::Grants(cmd) => run_grants(cmd).await,
    }
}

async fn run_secrets(cmd: SecretCommands) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        SecretCommands::List {
            resource_group,
            vault,
        } => secrets::list(&resource_group, &vault).await,
        SecretCommands::Set {
            resource_group,
            vault,
            name,
            value,
            expires,
        } => secrets::set(&resource_group, &vault, &name, value, expires).await,
        SecretCommands::Delete {
            resource_group,
            vault,
            name,
        } => secrets::delete(&resource_group, &vault, &name).await,
        SecretCommands::Restore {
            resource_group,
            vault,
            name,
            version,
        } => secrets::restore(&resource_group, &vault, &name, version).await,
        SecretCommands::Reveal {
            resource_group,
            vault,
            name,
            reason,
            raw,
        } => secrets::reveal(&resource_group, &vault, &name, &reason, raw).await,
    }
}

async fn run_grants(cmd: GrantCommands) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        GrantCommands::List {
            resource_group,
            vault,
        } => grants::list(&resource_group, &vault).await,
        GrantCommands::Add {
            resource_group,
            vault,
            who,
            can,
            on,
            approval,
        } => grants::add(&resource_group, &vault, &who, &can, on, approval).await,
        GrantCommands::Remove {
            resource_group,
            vault,
            id,
        } => grants::remove(&resource_group, &vault, &id).await,
    }
}
