pub mod create_user;
pub mod delete_user;
pub mod get_user;
pub mod info;
pub mod list_roles;
pub mod list_users;
pub mod set_role;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum TenantCommands {
    Info,
    Users,
    UserGet {
        id: String,
    },
    UserCreate {
        username: String,
        password: String,
        #[arg(long)]
        role: String,
        #[arg(long)]
        email: Option<String>,
        #[arg(long, default_value = "false")]
        force_password_change: bool,
    },
    UserDelete {
        id: String,
    },
    Roles,
    SetRole {
        #[arg(long)]
        user: String,
        #[arg(long)]
        role: String,
    },
}

pub async fn run(cmd: TenantCommands) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        TenantCommands::Info => info::run().await,
        TenantCommands::Users => list_users::run().await,
        TenantCommands::UserGet { id } => get_user::run(&id).await,
        TenantCommands::UserCreate {
            username,
            password,
            role,
            email,
            force_password_change,
        } => create_user::run(username, password, role, email, force_password_change).await,
        TenantCommands::UserDelete { id } => delete_user::run(&id).await,
        TenantCommands::Roles => list_roles::run().await,
        TenantCommands::SetRole { user, role } => set_role::run(&user, role).await,
    }
}
