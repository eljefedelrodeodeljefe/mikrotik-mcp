# Agent instructions

Guidance for AI coding agents (Claude Code, Codex, Cursor, etc.) working in
this repo. Human contributors: see `README.md`.

## What this is

MCP (Model Context Protocol) server for MikroTik RouterOS. Stdio transport
via [`rmcp`](https://crates.io/crates/rmcp); REST client via `reqwest`.
Rust 2024 edition (requires Rust 1.85+).

## Layout

```text
src/
├── main.rs      # binary entry point, stdio transport wiring, hot-reload watcher
├── server.rs    # MCP tool surface (declared with rmcp tool macros)
├── client.rs    # RouterOS REST client (reqwest + rustls)
├── params.rs    # serde/schemars input types for tools
└── error.rs     # thiserror error type
```

Add new MCP tools in `src/server.rs`. Add new RouterOS REST calls in
`src/client.rs`. Input/output schemas go in `src/params.rs` so `schemars`
can derive JSON Schema for the MCP tool catalog.

## Build / run

```sh
cargo build              # debug
cargo run                # launches stdio MCP server (expects MIKROTIK_* env vars)
cargo build --release    # release binary at target/release/mikrotik-mcp
```

The server reads RouterOS connection settings from env:

- `MIKROTIK_HOST` (required)
- `MIKROTIK_USER` + `MIKROTIK_PASSWORD` (option A)
- `MIKROTIK_API_TOKEN` (option B, takes precedence)

When iterating, run `bacon` in a second terminal — the server detects binary
mtime changes and exits, and the MCP client auto-restarts it. See `bacon.toml`.

## Before committing

All hooks must pass. Run via [`prek`][prek] (Rust reimplementation of
pre-commit — **do not use Python `pre-commit` or `uvx pre-commit`**):

```sh
prek run --all-files
```

Hooks: trailing-whitespace, end-of-file, check-toml/yaml, merge-conflict,
large-files, cspell, editorconfig-checker, markdownlint-cli2, `cargo fmt
--check`, `cargo clippy --all-targets -- -D warnings`.

Clippy is set to deny warnings — fix lints, don't `#[allow]` them away.
If `cspell` flags a real domain word, add it to `.cspell/project-words.txt`.

[prek]: https://github.com/j178/prek

## Branch naming (gitflow)

This repo follows [gitflow][gitflow]:

- `main` — production / released code. Tagged versions live here. Only
  release PRs (from release-please) and hotfixes merge directly.
- `develop` — integration branch. Day-to-day work targets this branch.
  Releases are cut by merging `develop` → `main`.
- `feature/<short-name>` — new features. Branch off `develop`, PR back into
  `develop`. Example: `feature/firewall-rules`.
- `bugfix/<short-name>` — non-urgent bug fixes. Branch off `develop`, PR into
  `develop`.
- `release/<version>` — release stabilisation. Branch off `develop`, PR into
  `main` (and back-merge into `develop`). Usually managed automatically by
  release-please.
- `hotfix/<short-name>` — urgent production fixes. Branch off `main`, PR into
  `main` *and* `develop`.

Use kebab-case slugs (`feature/dhcp-lease-tools`, not
`feature/DHCP_Lease_Tools`). Keep names short and topic-focused, not
ticket-numbered (we don't use issue trackers as filenames here).

CI runs on all five branch prefixes plus pushes to `main` and `develop`.

[gitflow]: https://nvie.com/posts/a-successful-git-branching-model/

## Commit messages

Conventional commits. The version is cut by release-please from these prefixes:

- `feat: …` → minor bump
- `fix: …`, `perf: …` → patch bump
- `feat!: …` or `BREAKING CHANGE:` footer → major bump
- `chore: …`, `ci: …`, `test: …`, `docs: …`, `refactor: …` → no version bump
  (some appear in the changelog, some are hidden — see
  `release-please-config.json`)

Keep one logical change per commit. Don't squash unrelated edits together.

## Releases

Don't bump the version manually. release-please opens a PR that bumps
`Cargo.toml`, `Cargo.lock`, `.release-please-manifest.json`, and the install
tag inside `README.md` (between the `x-release-please-start-version` HTML
markers). Merging that PR cuts the tag and triggers binary builds.

## CI

`.github/workflows/ci.yml` runs prek, check, fmt, clippy, test, and doc on
every push/PR to `main`. All jobs must pass. The release workflow
(`.github/workflows/release.yml`) runs separately and only acts on `main`
pushes or manual dispatch.

## Tool descriptions

Tool descriptions (the `description = "…"` string in each `#[tool(…)]` attribute)
must stay close to the language used in the official MikroTik documentation. Use
the same field names, menu paths, and terminology that RouterOS uses — don't invent
synonyms or over-explain RouterOS internals.

When adding or updating a tool, look up the relevant wiki page first and mirror
its terminology. Key references:

| Area | Wiki page |
|------|-----------|
| REST API general | <https://help.mikrotik.com/docs/spaces/ROS/pages/47579229/REST+API> |
| IP neighbors (CDP/LLDP/MNDP) | <https://help.mikrotik.com/docs/spaces/ROS/pages/8323118/IP+Neighbors> |
| IP addresses | <https://help.mikrotik.com/docs/spaces/ROS/pages/328088/IP+Address> |
| DHCP server | <https://help.mikrotik.com/docs/spaces/ROS/pages/24805500/DHCP> |
| Firewall filter & NAT | <https://help.mikrotik.com/docs/spaces/ROS/pages/328091/Firewall+Filter> |
| Interfaces | <https://help.mikrotik.com/docs/spaces/ROS/pages/328155/Ethernet> |
| Wireless (legacy) | <https://help.mikrotik.com/docs/spaces/ROS/pages/1409044/Wireless> |
| CAPsMAN | <https://help.mikrotik.com/docs/spaces/ROS/pages/1409149/CAPsMAN> |
| Routes | <https://help.mikrotik.com/docs/spaces/ROS/pages/328196/IP+Routing> |
| DNS | <https://help.mikrotik.com/docs/spaces/ROS/pages/24805404/DNS> |

## Things to avoid

- Don't write to stdout from the server process — stdout is the MCP protocol
  channel. Use `tracing` (stderr) for diagnostics.
- Don't add dependencies casually; this is a small surface. Justify in the
  commit message.
- Don't touch `Cargo.lock` by hand; let cargo manage it.
- Don't disable lints to make CI pass.
