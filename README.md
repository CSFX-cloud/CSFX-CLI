# CSF-CLI

Command-line interface for Cloud Service Foundry.

## Requirements

- Rust toolchain (stable) — [install via rustup](https://rustup.rs)

## Installation

```bash
git clone https://github.com/cs-foundry/CSF-CLI.git
cd CSF-CLI
cargo install --path cli
```

The `csf` binary is installed to `~/.cargo/bin/csf`. Ensure `~/.cargo/bin` is in your `PATH`.

## Update

```bash
git pull
cargo install --path cli --force
```
@
## Usage

```
csf                         # interactive REPL
csf login                   # authenticate against CSF-Core
csf logout                  # remove stored credentials
csf status                  # show current session info
csf token                   # print stored JWT token (useful for scripts)

csf volumes list
csf volumes get <id>
csf volumes snapshots
csf volumes nodes

csf registry agents
csf registry agents-get <id>
csf registry pre-register <name> <hostname> [--os <os>] [--arch <arch>] [--ttl <hours>]
csf registry pending
csf registry pending-delete <id>
csf registry deregister <id>
csf registry stats
csf registry tokens

csf nodes list
csf nodes metrics
```

## Configuration

Config is stored at `~/.csfx/config.json`:

```json
{
  "server": "http://localhost:8000",
  "token": "<jwt>"
}
```

## Scripting

```bash
export JWT=$(csf token)
GATEWAY_URL=http://localhost:8000 ./scripts/test-mtls.sh
```
