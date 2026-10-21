use colored::Colorize;
use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{CompletionType, Config, Context, Editor, Helper};
use std::borrow::Cow;

use crate::display;
use crate::events::EventCommands;
use crate::dns::{DnsCommands, RecordCommands};
use crate::gateway::GatewayCommands;
use crate::ingress::IngressCommands;
use crate::networks::NetworkCommands;
use crate::nodes::NodeCommands;
use crate::registry::RegistryCommands;
use crate::ssh::SshCommands;
use crate::system::SystemCommands;
use crate::tenant::TenantCommands;
use crate::volumes::VolumeCommands;
use crate::workloads::WorkloadCommands;

const COMMANDS: &[(&str, &str)] = &[
    ("login", "authenticate with the server"),
    ("setup", "initialize a fresh cluster"),
    ("logout", "clear authentication token"),
    ("status", "show session and user info"),
    ("volumes list", "list all volumes"),
    ("volumes get <id>", "show volume details"),
    ("volumes create <name> --size <gb>", "create a new volume"),
    ("volumes delete <id>", "delete a volume"),
    ("volumes attach <id> --agent <agent-id>", "attach volume to agent"),
    ("volumes detach <id>", "detach volume from agent"),
    ("volumes snapshots --volume <id>", "list snapshots of a volume"),
    ("volumes snapshot-create --volume <id> --name <name>", "create a snapshot"),
    ("registry agents", "list registered agents"),
    ("registry agents-get <id>", "show agent details"),
    ("registry deregister <id>", "deregister an agent"),
    ("registry stats", "show registry statistics"),
    ("registry bootstrap-create", "create a cluster-wide bootstrap token"),
    ("registry bootstrap-list", "list active bootstrap tokens"),
    ("registry bootstrap-revoke <id>", "revoke a bootstrap token"),
    ("nodes list", "list all nodes"),
    ("nodes get <id>", "show node details"),
    ("nodes metrics", "show cluster node metrics"),
    ("nodes metrics --watch", "live-refresh cluster node metrics"),
    ("nodes agent-metrics <id>", "show metrics for a specific agent"),
    ("nodes agent-metrics <id> --watch", "live-refresh agent metrics"),
    ("system stats", "show cluster-wide aggregated stats"),
    ("system stats --watch", "live-refresh cluster stats"),
    ("system update <version>", "schedule and watch a control plane update"),
    ("system update-watch", "watch current update progress"),
    ("system update-pause", "pause automatic updates"),
    ("system update-resume", "resume automatic updates"),
    ("system check-update", "check for available releases"),
    ("workloads list", "list all workloads"),
    ("workloads get <id>", "show workload details"),
    ("workloads create <name> <image>", "schedule a new workload"),
    ("workloads delete <id>", "delete a workload"),
    ("workloads metrics <id>", "show workload resource usage"),
    ("workloads metrics <id> --watch", "live-refresh workload metrics"),
    ("events list", "list failover and audit events"),
    ("networks list", "list all overlay networks"),
    ("networks get <id>", "show network details"),
    ("networks create <name> <cidr>", "create a new overlay network"),
    ("networks delete <id>", "delete a network"),
    ("networks policies --network <id>", "list network policies"),
    ("networks policy-create --network <id> --direction <in|out> --action <allow|deny>", "create a network policy"),
    ("networks members --network <id>", "list network members"),
    ("networks member-add --network <id> --workload <id>", "add workload to network"),
    ("networks member-remove --network <id> --workload <id>", "remove workload from network"),
    ("tenant info", "show organization details"),
    ("tenant users", "list users in the organization"),
    ("tenant user-get <id>", "show user details"),
    ("tenant user-create <username> <password> --role <role-id>", "create a new user"),
    ("tenant user-delete <id>", "delete a user"),
    ("tenant roles", "list available roles"),
    ("tenant set-role --user <id> --role <role-id>", "assign a role to a user"),
    ("ssh connect <node>", "open ssh session to a cluster node"),
    ("ssh enable [--hours <n> | --unlimited]", "allow ssh access on all nodes"),
    ("ssh disable", "close ssh access on all nodes"),
    ("ssh status", "show whether ssh access is open"),
    ("clear", "clear the terminal screen"),
    ("help", "show available commands"),
    ("exit", "exit the shell"),
];

struct CsfHelper;

impl Completer for CsfHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let prefix = &line[..pos];
        let candidates: Vec<Pair> = COMMANDS
            .iter()
            .filter(|(cmd, _)| cmd.starts_with(prefix))
            .map(|(cmd, desc)| Pair {
                display: format!("{:<56} {}", cmd, desc.dimmed()),
                replacement: cmd.to_string(),
            })
            .collect();
        Ok((0, candidates))
    }
}

impl Hinter for CsfHelper {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> Option<String> {
        if pos < line.len() {
            return None;
        }
        COMMANDS
            .iter()
            .find(|(cmd, _)| cmd.starts_with(line) && *cmd != line)
            .map(|(cmd, _)| cmd[line.len()..].to_string())
    }
}

impl Highlighter for CsfHelper {
    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Cow::Owned(hint.dimmed().to_string())
    }
}

impl Validator for CsfHelper {}

impl Helper for CsfHelper {}

fn print_help() {
    display::section("Commands");

    let groups: &[(&str, &[&str])] = &[
        ("Auth", &["login", "setup", "logout", "status"]),
        (
            "Volumes",
            &[
                "volumes list",
                "volumes get <id>",
                "volumes create <name> --size <gb>",
                "volumes delete <id>",
                "volumes attach <id> --agent <agent-id>",
                "volumes detach <id>",
                "volumes snapshots --volume <id>",
                "volumes snapshot-create --volume <id> --name <name>",
            ],
        ),
        (
            "Registry",
            &[
                "registry agents",
                "registry agents-get <id>",
                "registry deregister <id>",
                "registry stats",
                "registry bootstrap-create",
                "registry bootstrap-list",
                "registry bootstrap-revoke <id>",
            ],
        ),
        (
            "Nodes",
            &[
                "nodes list",
                "nodes get <id>",
                "nodes join-token [worker|master]",
                "nodes cordon <id>",
                "nodes uncordon <id>",
                "nodes drain <id>",
                "nodes remove <id>",
                "nodes metrics",
                "nodes metrics --watch",
                "nodes agent-metrics <id>",
                "nodes agent-metrics <id> --watch",
            ],
        ),
        (
            "System",
            &[
                "system stats",
                "system stats --watch",
                "system update <version>",
                "system update-watch",
                "system update-pause",
                "system update-resume",
                "system check-update",
            ],
        ),
        (
            "Workloads",
            &[
                "workloads list",
                "workloads get <id>",
                "workloads create <name> <image>",
                "workloads delete <id>",
                "workloads metrics <id>",
                "workloads metrics <id> --watch",
            ],
        ),
        ("Events", &["events list"]),
        (
            "Gateways",
            &[
                "gateway list <resource-group>",
                "gateway create <resource-group> <name> [--http-port <port>] [--https-port <port>]",
                "gateway delete <resource-group> <id>",
            ],
        ),
        (
            "Ingress",
            &[
                "ingress list <resource-group>",
                "ingress add <resource-group> <hostname> <workload> <port> --gateway <id>",
                "ingress delete <resource-group> <id>",
            ],
        ),
        (
            "DNS",
            &[
                "dns zone",
                "dns trust",
                "dns record list <resource-group>",
                "dns record add <resource-group> <name> <A|CNAME> <value> [--ttl <seconds>]",
                "dns record delete <resource-group> <id>",
            ],
        ),
        (
            "Networks",
            &[
                "networks list",
                "networks get <id>",
                "networks create <name> <cidr>",
                "networks delete <id>",
                "networks policies --network <id>",
                "networks policy-create --network <id> --direction <in|out> --action <allow|deny>",
                "networks members --network <id>",
                "networks member-add --network <id> --workload <id>",
                "networks member-remove --network <id> --workload <id>",
            ],
        ),
        (
            "Tenant",
            &[
                "tenant info",
                "tenant users",
                "tenant user-get <id>",
                "tenant user-create <username> <password> --role <role-id>",
                "tenant user-delete <id>",
                "tenant roles",
                "tenant set-role --user <id> --role <role-id>",
            ],
        ),
        (
            "SSH",
            &[
                "ssh connect <node>",
                "ssh enable [--hours <n> | --unlimited]",
                "ssh disable",
                "ssh status",
            ],
        ),
        ("Shell", &["clear", "help", "exit"]),
    ];

    for (group, cmds) in groups {
        println!("  {}", group.bold().cyan());
        for cmd in *cmds {
            let base = cmd.split_whitespace().take(2).collect::<Vec<_>>().join(" ");
            if let Some((_, desc)) = COMMANDS.iter().find(|(c, _)| c.starts_with(&base)) {
                println!("    {:<56} {}", cmd.bold(), desc.dimmed());
            }
        }
    }
    println!();
}

fn parse_flag<'a>(parts: &[&'a str], flag: &str) -> Option<&'a str> {
    parts
        .windows(2)
        .find(|w| w[0] == flag)
        .map(|w| w[1])
}

async fn dispatch(parts: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    match parts {
        ["login"] => crate::user::login::login().await?,
        ["setup"] => crate::user::setup::setup(None).await?,
        ["setup", server] => crate::user::setup::setup(Some(server.to_string())).await?,
        ["logout"] => crate::user::logout::logout().await?,
        ["status"] => crate::user::status::status().await?,

        ["volumes", "list"] => crate::volumes::run(VolumeCommands::List).await?,
        ["volumes", "get", id] => {
            crate::volumes::run(VolumeCommands::Get { id: id.to_string() }).await?
        }
        ["volumes", "create", name, "--size", size] => {
            crate::volumes::run(VolumeCommands::Create {
                name: name.to_string(),
                size: size.parse().unwrap_or(10),
                pool: None,
            })
            .await?
        }
        ["volumes", "create", name, "--size", size, "--pool", pool] => {
            crate::volumes::run(VolumeCommands::Create {
                name: name.to_string(),
                size: size.parse().unwrap_or(10),
                pool: Some(pool.to_string()),
            })
            .await?
        }
        ["volumes", "delete", id] => {
            crate::volumes::run(VolumeCommands::Delete { id: id.to_string() }).await?
        }
        ["volumes", "attach", id, "--agent", agent] => {
            crate::volumes::run(VolumeCommands::Attach {
                id: id.to_string(),
                agent: agent.to_string(),
                workload: None,
            })
            .await?
        }
        ["volumes", "attach", id, "--agent", agent, "--workload", wid] => {
            crate::volumes::run(VolumeCommands::Attach {
                id: id.to_string(),
                agent: agent.to_string(),
                workload: Some(wid.to_string()),
            })
            .await?
        }
        ["volumes", "detach", id] => {
            crate::volumes::run(VolumeCommands::Detach { id: id.to_string() }).await?
        }
        ["volumes", "snapshots", "--volume", vid] => {
            crate::volumes::run(VolumeCommands::Snapshots { volume: vid.to_string() }).await?
        }
        ["volumes", "snapshot-create", "--volume", vid, "--name", name] => {
            crate::volumes::run(VolumeCommands::SnapshotCreate {
                volume: vid.to_string(),
                name: name.to_string(),
            })
            .await?
        }

        ["registry", "agents"] => crate::registry::run(RegistryCommands::Agents).await?,
        ["registry", "agents-get", id] => {
            crate::registry::run(RegistryCommands::AgentsGet { id: id.to_string() }).await?
        }
        ["registry", "deregister", id] => {
            crate::registry::run(RegistryCommands::Deregister { id: id.to_string() }).await?
        }
        ["registry", "stats"] => crate::registry::run(RegistryCommands::Stats).await?,
        ["registry", "bootstrap-create"] => {
            crate::registry::run(RegistryCommands::BootstrapCreate {
                description: None,
                ttl: None,
                max_uses: None,
            })
            .await?
        }
        ["registry", "bootstrap-list"] => {
            crate::registry::run(RegistryCommands::BootstrapList).await?
        }
        ["registry", "bootstrap-revoke", id] => {
            crate::registry::run(RegistryCommands::BootstrapRevoke { id: id.to_string() }).await?
        }

        ["nodes", "list"] => crate::nodes::run(NodeCommands::List).await?,
        ["nodes", "get", id] => {
            crate::nodes::run(NodeCommands::Get { id: id.to_string() }).await?
        }
        ["nodes", "join-token"] => {
            crate::nodes::run(NodeCommands::JoinToken { role: "worker".to_string(), ttl: 1 }).await?
        }
        ["nodes", "join-token", role @ ("worker" | "master")] => {
            crate::nodes::run(NodeCommands::JoinToken { role: role.to_string(), ttl: 1 }).await?
        }
        ["nodes", "cordon", id] => {
            crate::nodes::run(NodeCommands::Cordon { id: id.to_string() }).await?
        }
        ["nodes", "uncordon", id] => {
            crate::nodes::run(NodeCommands::Uncordon { id: id.to_string() }).await?
        }
        ["nodes", "drain", id] => {
            crate::nodes::run(NodeCommands::Drain { id: id.to_string() }).await?
        }
        ["nodes", "remove", id] => {
            crate::nodes::run(NodeCommands::Remove { id: id.to_string() }).await?
        }
        ["nodes", "metrics"] => {
            crate::nodes::run(NodeCommands::Metrics { watch: false }).await?
        }
        ["nodes", "metrics", "--watch"] => {
            crate::nodes::run(NodeCommands::Metrics { watch: true }).await?
        }
        ["nodes", "agent-metrics", id] => {
            crate::nodes::run(NodeCommands::AgentMetrics { id: id.to_string(), watch: false }).await?
        }
        ["nodes", "agent-metrics", id, "--watch"] => {
            crate::nodes::run(NodeCommands::AgentMetrics { id: id.to_string(), watch: true }).await?
        }

        ["system", "stats"] => {
            crate::system::run(SystemCommands::Stats { watch: false }).await?
        }
        ["system", "stats", "--watch"] => {
            crate::system::run(SystemCommands::Stats { watch: true }).await?
        }
        ["system", "update", version] => {
            crate::system::run(SystemCommands::Update { version: version.to_string() }).await?
        }
        ["system", "update-watch"] => {
            crate::system::run(SystemCommands::UpdateWatch).await?
        }
        ["system", "update-pause"] => {
            crate::system::run(SystemCommands::UpdatePause).await?
        }
        ["system", "update-resume"] => {
            crate::system::run(SystemCommands::UpdateResume).await?
        }
        ["system", "check-update"] => {
            crate::system::run(SystemCommands::CheckUpdate { pre: false }).await?
        }
        ["system", "check-update", "--pre"] => {
            crate::system::run(SystemCommands::CheckUpdate { pre: true }).await?
        }

        ["workloads", "metrics", id] => {
            crate::workloads::run(WorkloadCommands::Metrics { id: id.to_string(), watch: false }).await?
        }
        ["workloads", "metrics", id, "--watch"] => {
            crate::workloads::run(WorkloadCommands::Metrics { id: id.to_string(), watch: true }).await?
        }
        ["workloads", "list"] => crate::workloads::run(WorkloadCommands::List).await?,
        ["workloads", "create", name, image] => {
            crate::workloads::run(WorkloadCommands::Create {
                name: name.to_string(),
                image: image.to_string(),
                cpu: 500,
                memory: 536870912,
                disk: 10737418240,
            })
            .await?
        }
        ["workloads", "create", name, image, "--cpu", cpu] => {
            crate::workloads::run(WorkloadCommands::Create {
                name: name.to_string(),
                image: image.to_string(),
                cpu: cpu.parse().unwrap_or(500),
                memory: 536870912,
                disk: 10737418240,
            })
            .await?
        }
        ["workloads", "get", id] => {
            crate::workloads::run(WorkloadCommands::Get { id: id.to_string() }).await?
        }
        ["workloads", "delete", id] => {
            crate::workloads::run(WorkloadCommands::Delete { id: id.to_string() }).await?
        }

        ["events", "list"] => crate::events::run(EventCommands::List).await?,

        ["ingress", "list", group] => {
            crate::ingress::run(IngressCommands::List {
                resource_group: group.to_string(),
            })
            .await?
        }
        ["ingress", "add", group, hostname, workload, port, ..] => {
            let gateway = parse_flag(parts, "--gateway").ok_or("missing --gateway <id>")?;
            crate::ingress::run(IngressCommands::Add {
                resource_group: group.to_string(),
                hostname: hostname.to_string(),
                workload: workload.to_string(),
                port: port.parse()?,
                gateway: gateway.to_string(),
                no_tls: false,
            })
            .await?
        }
        ["ingress", "delete", group, id] => {
            crate::ingress::run(IngressCommands::Delete {
                resource_group: group.to_string(),
                id: id.to_string(),
            })
            .await?
        }

        ["gateway", "list", group] => {
            crate::gateway::run(GatewayCommands::List {
                resource_group: group.to_string(),
            })
            .await?
        }
        ["gateway", "create", group, name, ..] => {
            crate::gateway::run(GatewayCommands::Create {
                resource_group: group.to_string(),
                name: name.to_string(),
                http_port: parse_flag(parts, "--http-port").map_or(Ok(80), str::parse)?,
                https_port: parse_flag(parts, "--https-port").map_or(Ok(443), str::parse)?,
            })
            .await?
        }
        ["gateway", "delete", group, id] => {
            crate::gateway::run(GatewayCommands::Delete {
                resource_group: group.to_string(),
                id: id.to_string(),
            })
            .await?
        }

        ["dns", "zone"] => crate::dns::run(DnsCommands::Zone).await?,
        ["dns", "trust"] => crate::dns::run(DnsCommands::Trust).await?,
        ["dns", "record", "list", group] => {
            crate::dns::run(DnsCommands::Record(RecordCommands::List {
                resource_group: group.to_string(),
            }))
            .await?
        }
        ["dns", "record", "add", group, name, record_type, value, ..] => {
            crate::dns::run(DnsCommands::Record(RecordCommands::Add {
                resource_group: group.to_string(),
                name: name.to_string(),
                record_type: record_type.to_string(),
                value: value.to_string(),
                ttl: parse_flag(parts, "--ttl").map(str::parse).transpose()?,
            }))
            .await?
        }
        ["dns", "record", "delete", group, id] => {
            crate::dns::run(DnsCommands::Record(RecordCommands::Delete {
                resource_group: group.to_string(),
                id: id.to_string(),
            }))
            .await?
        }

        ["networks", "list"] => crate::networks::run(NetworkCommands::List).await?,
        ["networks", "get", id] => {
            crate::networks::run(NetworkCommands::Get { id: id.to_string() }).await?
        }
        ["networks", "delete", id] => {
            crate::networks::run(NetworkCommands::Delete { id: id.to_string() }).await?
        }
        ["networks", "create", name, cidr] => {
            crate::networks::run(NetworkCommands::Create {
                name: name.to_string(),
                cidr: cidr.to_string(),
                overlay: "wireguard".to_string(),
            })
            .await?
        }
        ["networks", "create", name, cidr, "--overlay", overlay] => {
            crate::networks::run(NetworkCommands::Create {
                name: name.to_string(),
                cidr: cidr.to_string(),
                overlay: overlay.to_string(),
            })
            .await?
        }
        parts if parts.first() == Some(&"networks") && parts.get(1) == Some(&"policies") => {
            let network = parse_flag(parts, "--network").unwrap_or("").to_string();
            crate::networks::run(NetworkCommands::Policies { network }).await?
        }
        parts if parts.first() == Some(&"networks") && parts.get(1) == Some(&"policy-create") => {
            let network = parse_flag(parts, "--network").unwrap_or("").to_string();
            let direction = parse_flag(parts, "--direction").unwrap_or("ingress").to_string();
            let action = parse_flag(parts, "--action").unwrap_or("allow").to_string();
            let priority = parse_flag(parts, "--priority").and_then(|v| v.parse().ok()).unwrap_or(100);
            let source = parse_flag(parts, "--source").map(|s| s.to_string());
            let destination = parse_flag(parts, "--destination").map(|s| s.to_string());
            let port = parse_flag(parts, "--port").and_then(|v| v.parse().ok());
            let protocol = parse_flag(parts, "--protocol").map(|s| s.to_string());
            crate::networks::run(NetworkCommands::PolicyCreate {
                network, direction, action, priority, source, destination, port, protocol,
            })
            .await?
        }
        parts if parts.first() == Some(&"networks") && parts.get(1) == Some(&"members") => {
            let network = parse_flag(parts, "--network").unwrap_or("").to_string();
            crate::networks::run(NetworkCommands::Members { network }).await?
        }
        parts if parts.first() == Some(&"networks") && parts.get(1) == Some(&"member-add") => {
            let network = parse_flag(parts, "--network").unwrap_or("").to_string();
            let workload = parse_flag(parts, "--workload").unwrap_or("").to_string();
            crate::networks::run(NetworkCommands::MemberAdd { network, workload }).await?
        }
        parts if parts.first() == Some(&"networks") && parts.get(1) == Some(&"member-remove") => {
            let network = parse_flag(parts, "--network").unwrap_or("").to_string();
            let workload = parse_flag(parts, "--workload").unwrap_or("").to_string();
            crate::networks::run(NetworkCommands::MemberRemove { network, workload }).await?
        }

        ["tenant", "info"] => crate::tenant::run(TenantCommands::Info).await?,
        ["tenant", "users"] => crate::tenant::run(TenantCommands::Users).await?,
        ["tenant", "user-get", id] => {
            crate::tenant::run(TenantCommands::UserGet { id: id.to_string() }).await?
        }
        ["tenant", "user-delete", id] => {
            crate::tenant::run(TenantCommands::UserDelete { id: id.to_string() }).await?
        }
        ["tenant", "roles"] => crate::tenant::run(TenantCommands::Roles).await?,
        parts if parts.first() == Some(&"tenant") && parts.get(1) == Some(&"user-create") => {
            if parts.len() < 4 {
                display::error("usage: tenant user-create <username> <password> --role <role-id>");
            } else {
                let username = parts[2].to_string();
                let password = parts[3].to_string();
                let role = parse_flag(parts, "--role").unwrap_or("").to_string();
                let email = parse_flag(parts, "--email").map(|s| s.to_string());
                let force = parse_flag(parts, "--force-password-change")
                    .map(|v| v == "true")
                    .unwrap_or(false);
                crate::tenant::run(TenantCommands::UserCreate {
                    username,
                    password,
                    role,
                    email,
                    force_password_change: force,
                })
                .await?
            }
        }
        parts if parts.first() == Some(&"tenant") && parts.get(1) == Some(&"set-role") => {
            let user = parse_flag(parts, "--user").unwrap_or("").to_string();
            let role = parse_flag(parts, "--role").unwrap_or("").to_string();
            crate::tenant::run(TenantCommands::SetRole { user, role }).await?
        }

        ["ssh", "connect", node] => {
            crate::ssh::run(SshCommands::Connect {
                node: node.to_string(),
                port: 22,
            })
            .await?
        }
        ["ssh", "enable", "--unlimited"] => {
            crate::ssh::run(SshCommands::Enable {
                hours: None,
                unlimited: true,
            })
            .await?
        }
        ["ssh", "enable", "--hours", hours] => {
            crate::ssh::run(SshCommands::Enable {
                hours: hours.parse().ok(),
                unlimited: false,
            })
            .await?
        }
        ["ssh", "enable"] => {
            crate::ssh::run(SshCommands::Enable {
                hours: None,
                unlimited: false,
            })
            .await?
        }
        ["ssh", "disable"] => crate::ssh::run(SshCommands::Disable).await?,
        ["ssh", "status"] => crate::ssh::run(SshCommands::Status).await?,
        ["ssh", "connect", node, "--port", port] => {
            crate::ssh::run(SshCommands::Connect {
                node: node.to_string(),
                port: port.parse().unwrap_or(22),
            })
            .await?
        }

        ["clear"] => {
            print!("\x1B[2J\x1B[H");
            let _ = std::io::Write::flush(&mut std::io::stdout());
        }
        ["help"] | ["?"] => print_help(),

        [cmd, ..] => {
            display::error(&format!("unknown command: {}", cmd));
            display::info("type 'help' to see available commands");
        }
        [] => {}
    }
    Ok(())
}

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::builder()
        .completion_type(CompletionType::List)
        .build();

    let mut rl: Editor<CsfHelper, _> = Editor::with_config(config)?;
    rl.set_helper(Some(CsfHelper));

    let history_path = dirs::home_dir().map(|mut p| {
        p.push(".csf");
        let _ = std::fs::create_dir_all(&p);
        p.push("history");
        p
    });

    if let Some(ref path) = history_path {
        let _ = rl.load_history(path);
    }

    println!(
        "  {}",
        "Tab for completion, arrow keys for history, exit to quit".dimmed()
    );
    println!();

    loop {
        let prompt = format!("{} ", "csfx>".truecolor(220, 220, 220));

        match rl.readline(&prompt) {
            Ok(line) => {
                let trimmed = line.trim().to_string();
                if trimmed.is_empty() {
                    continue;
                }

                let _ = rl.add_history_entry(trimmed.as_str());

                if matches!(trimmed.as_str(), "exit" | "quit" | "q") {
                    break;
                }

                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if let Err(e) = dispatch(&parts).await {
                    display::error(&e.to_string());
                }
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                println!();
                break;
            }
            Err(e) => {
                display::error(&format!("readline error: {}", e));
                break;
            }
        }
    }

    if let Some(ref path) = history_path {
        let _ = rl.save_history(path);
    }

    Ok(())
}
