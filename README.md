# mikrotik-mcp

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

```sh
cargo install --git https://github.com/eljefedelrodeodeljefe/mikrotik-mcp.git \
  --tag v0.1.0 # x-release-please-version
```

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

Releases are cut by [release-please][rp] using conventional-commit messages
on `main`. The workflow runs on every push to `main` and maintains a release
PR; merging that PR tags + creates a GitHub release and uploads prebuilt
binaries (Linux/macOS, x86_64 + arm64).

Trigger manually with **Actions → Release → Run workflow** (workflow_dispatch).

Commit prefixes that affect the version:

- `feat: …` → minor bump
- `fix: …`, `perf: …` → patch bump
- `feat!: …` or `BREAKING CHANGE:` footer → major bump

[rp]: https://github.com/googleapis/release-please

## License

MIT
