# mikrotik-mcp

> ⚠️ **Experimental.** APIs, tool surface, and config may change without
> notice. That said, this is **actively used and actively developed** —
> issues and PRs welcome, breakage will be fixed quickly.

MCP (Model Context Protocol) server for MikroTik RouterOS. Exposes RouterOS
operations as MCP tools so AI agents can query and manage MikroTik devices.

## Status

Initial implementation: stdio MCP transport (`rmcp`), RouterOS REST client
(`reqwest`), tool surface in `src/server.rs`.

## Install

### From source

Requires Rust (stable). Install via [rustup](https://rustup.rs/).

```sh
git clone https://github.com/eljefedelrodeodeljefe/mikrotik-mcp.git
cd mikrotik-mcp
cargo build --release
```

The binary lands at `target/release/mikrotik-mcp`.

### With cargo install

<!-- x-release-please-start-version -->
```sh
cargo install --git https://github.com/eljefedelrodeodeljefe/mikrotik-mcp.git \
  --tag v0.3.0
```
<!-- x-release-please-end -->

This places `mikrotik-mcp` in `~/.cargo/bin/`.

## Configuration

The server authenticates to RouterOS using **either** a username/password pair
**or** an API token. Provide credentials through environment variables — never
commit them.

### Required

| Variable | Description |
|----------|-------------|
| `MIKROTIK_HOST` | RouterOS host, e.g. `192.168.88.1` or `router.lan:8728` |

### Auth — option A: user + password

| Variable | Description |
|----------|-------------|
| `MIKROTIK_USER` | RouterOS username |
| `MIKROTIK_PASSWORD` | RouterOS password |

### Auth — option B: API token

| Variable | Description |
|----------|-------------|
| `MIKROTIK_API_TOKEN` | RouterOS REST API token |

If both are set, the API token takes precedence.

### Optional

| Variable | Default | Description |
|----------|---------|-------------|
| `MIKROTIK_PORT` | `443` | RouterOS REST API port |
| `MIKROTIK_TLS_VERIFY` | `false` | Verify TLS certificate (`true`/`false`) |
| `MIKROTIK_BACKUP_ENCRYPT` | `true` | Encrypt backups with `MIKROTIK_PASSWORD` |
| `MIKROTIK_ALLOW_WRITES` | `false` | Enable mutating tools |

The client picks HTTP for port 80 and HTTPS otherwise. If the device only
has `www` (plain HTTP) enabled rather than `www-ssl`, set `MIKROTIK_PORT=80`.

`MIKROTIK_ALLOW_WRITES` is a convenience guard — it is not a security
boundary. For genuine read-only enforcement, use a RouterOS user with
`policy=read,api,rest-api` and no write policy.

## Injecting the secret

### Inline env

```sh
MIKROTIK_HOST=192.168.88.1 \
MIKROTIK_USER=admin \
MIKROTIK_PASSWORD='changeme' \
mikrotik-mcp
```

### `.env` file (local dev)

```sh
cp .env.example .env
$EDITOR .env
set -a; source .env; set +a
mikrotik-mcp
```

`.env` is `.gitignore`d.

### Claude Code / Claude Desktop MCP config

Register the server and pass secrets via the `env` field:

```json
{
  "mcpServers": {
    "mikrotik": {
      "command": "mikrotik-mcp",
      "env": {
        "MIKROTIK_HOST": "192.168.88.1",
        "MIKROTIK_API_TOKEN": "..."
      }
    }
  }
}
```

Or via the Claude Code CLI:

```sh
claude mcp add mikrotik mikrotik-mcp \
  -e MIKROTIK_HOST=192.168.88.1 \
  -e MIKROTIK_API_TOKEN=...
```

#### Running from a cargo checkout

If you haven't installed the binary, point the config at `cargo` and run from
the source tree:

```json
{
  "mcpServers": {
    "mikrotik": {
      "command": "cargo",
      "args": [
        "run",
        "--quiet",
        "--release",
        "--manifest-path",
        "/abs/path/to/mikrotik-mcp/Cargo.toml"
      ],
      "env": {
        "MIKROTIK_HOST": "192.168.88.1",
        "MIKROTIK_API_TOKEN": "..."
      }
    }
  }
}
```

`--quiet` keeps cargo's build chatter off stdout (MCP uses stdout for the
protocol). The CLI equivalent:

```sh
claude mcp add mikrotik cargo \
  -e MIKROTIK_HOST=192.168.88.1 \
  -e MIKROTIK_API_TOKEN=... \
  -- run --quiet --release --manifest-path /abs/path/to/mikrotik-mcp/Cargo.toml
```

#### Hot reload with bacon

A `bacon.toml` is included. With Claude Code wired to `cargo run` (above),
keep a `bacon` instance running in the repo for hot reload:

```sh
cargo install bacon   # one-time
bacon                  # default job: cargo build
```

Workflow:

1. Claude Code launches the MCP server via `cargo run`.
2. `bacon` watches `src/` and rebuilds on save.
3. The rebuild swaps the binary → the running server exits → Claude Code
   auto-restarts it with the new build. No Claude restart needed.

Other jobs: `bacon check`, `bacon clippy`.

## eSIM (eUICC) provisioning

Tools are provided to manage eSIM profiles on modems with an eUICC chip
(e.g. the Chateau 5G R17 ax / Quectel RG650E): `set_lte_sim_slot`,
`list_esim_profiles`, `get_esim_id`, `provision_esim`,
`activate_esim_profile`, `deactivate_esim_profile`, `remove_esim_profile`.

Typical flow: `set_lte_sim_slot esim` → `provision_esim` → `list_esim_profiles`.

> [!IMPORTANT]
> **`provision_esim` cannot complete a profile install over REST.** RouterOS
> gates the final download behind an interactive end-user consent (the `y/N`
> prompt you answer in a CLI terminal), and the REST API has no parameter to
> supply it. A REST `provision` runs through SM-DP+ authentication and staging,
> then ends with `user didn't approve`. Worse, that staged-but-unapproved
> session **advances the profile's state on the SM-DP+** and can strand it in a
> non-`Released` state — subsequent attempts then fail with
> `Bad profile state (reason code 1.2)` until the server session expires or the
> operator releases the profile.
>
> To actually download/install a profile, run it from a router terminal and
> press `y`:
>
> ```text
> /interface/lte/esim provision interface=lte1 \
>   sm-dp-plus="<sm-dp+ host>" matching-id="<activation code>"
> ```
>
> `provision_esim` is therefore intended for **staging/diagnostics**: it detects
> the `user didn't approve` and `Bad profile state` outcomes and returns the
> SM-DP+ `subjectCode`/`reasonCode`/`message` with guidance, rather than
> reporting a half-finished stream as success.

Some modems also do not answer `esim-id` (EID query) over REST and will return
an error status for `get_esim_id`; this is a modem/firmware limitation, not a
transport bug.

## WAN failover (LTE as backup)

Tools to wire LTE up as a backup WAN behind a wired primary:
`list_dhcp_clients` / `set_dhcp_client` (the `set` tool locates the client by
interface), and `list_lte_apn_profiles` / `set_lte_apn_profile` (locates the
profile by name, or the default profile). Combined with `add_route`
(`check_gateway=ping`, `distance`) and the `WAN` interface list (which the
masquerade rule already follows), a link/gateway failover looks like:

1. Primary `ether1` default route at distance 1 with `check_gateway=ping`
   (`add_route`); stop the DHCP client adding its own un-checked route:
   `set_dhcp_client interface=ether1 add_default_route=false`.
2. LTE default route at distance 2 — either `add_route gateway=lte1 distance=2`,
   or `set_lte_apn_profile default_route_distance=2`.

ether1 is used while its gateway answers ping; when it stops, that route goes
inactive and LTE (distance 2) takes over, reclaiming once ether1 recovers.

## Wi-Fi (APs, virtual APs, station VIFs)

`list_wifi_interfaces` / `scan_wifi`, security profiles
(`list_wifi_security` / `add_wifi_security`), and interface lifecycle
(`set_wifi_interface` / `remove_wifi_interface`). Two VIF builders sit on top of
an existing radio (`master-interface`):

- `add_wifi_station` — a station VIF that connects upstream (e.g. wifi3 on wifi1
  to a phone hotspot as a tertiary WAN).
- `add_wifi_ap` — an extra AP SSID (second BSSID) on the same radio. Reference
  the master's security profile to make it a true alias — edit the profile once
  and both SSIDs update.

Give a new AP SSID LAN access with the bridge-port tools
(`list_bridge_ports` / `add_bridge_port` / `remove_bridge_port`). To add a second
"Spine" SSID aliasing an existing 2.4 GHz AP on `wifi2`:

1. `add_wifi_ap name=spine master_interface=wifi2 ssid=Spine security="Quick Set"`
2. `add_bridge_port bridge=bridge interface=spine pvid=1`

A virtual AP's interface `running` flag stays `false` until a client associates
even while it is beaconing — confirm it's up via the radio state, not that flag.

The `list_files` / `remove_file` tools round out backup hygiene: after
`save_backup` downloads a `.backup` locally, `remove_file` clears the on-device
copy.

## Development

Pre-commit hooks (rustfmt, clippy, editorconfig, markdownlint, basic
hygiene) live in `.pre-commit-config.yaml`. Run them with [prek][prek],
the Rust reimplementation of pre-commit:

```sh
prek install            # one-time: install git hook
prek run --all-files    # run all hooks against the whole tree
```

[prek]: https://github.com/j178/prek

## Releases

Releases are cut by [release-please][rp] using conventional-commit messages.
Two branches feed it:

- **`main`** — stable releases (`v0.2.0`, `v0.2.1`, …). Merging the
  release PR tags + creates a GitHub release and uploads prebuilt binaries
  (Linux/macOS, x86_64 + arm64).
- **`develop`** — pre-releases (`v0.2.0-rc.1`, `v0.2.0-rc.2`, …). Tags are
  cut and GH releases are marked "pre-release", but **no binaries are
  uploaded** for prereleases — install from source or use `cargo install
  --git ... --tag v0.2.0-rc.1` if you need to try one.

Trigger manually with **Actions → Release → Run workflow** (workflow_dispatch).

Commit prefixes that affect the version:

- `feat: …` → minor bump
- `fix: …`, `perf: …` → patch bump
- `feat!: …` or `BREAKING CHANGE:` footer → major bump

[rp]: https://github.com/googleapis/release-please

## License

MIT
