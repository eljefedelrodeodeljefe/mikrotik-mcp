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
├── main.rs          # binary entry point, stdio transport wiring, hot-reload watcher
├── server.rs        # thin MCP adapter — one #[tool] stub per tool, write gate
├── client.rs        # RouterOS REST client (reqwest + rustls)
├── error.rs         # error helpers
├── params/          # serde/schemars input structs, one file per domain
│   ├── mod.rs
│   ├── shared.rs    # RemoveByIdParams (used by every remove_ tool)
│   ├── system.rs
│   ├── interfaces.rs
│   ├── ip.rs
│   ├── firewall.rs
│   ├── dhcp.rs
│   └── dns.rs
└── tools/           # pure RouterOS logic — no MCP types, fully testable
    ├── mod.rs
    ├── system.rs
    ├── interfaces.rs
    ├── ip.rs
    ├── firewall.rs
    ├── dhcp.rs
    ├── dns.rs
    └── network.rs   # routes + neighbor discovery
```

**Adding a new tool:**

1. Add a param struct to `src/params/{domain}.rs` (derive `Deserialize +
   JsonSchema`).
2. Add a pure function to `src/tools/{domain}.rs`. It takes `&RouterosClient`
   and any params and returns `anyhow::Result<Value>`. Add a wiremock test.
3. Add a short `#[tool(description = "…")]` stub in the matching `impl`
   block in `src/server.rs`. Call `self.guard_write()?` first for any
   mutating tool. A read-only tool's name starts with `get_`, `list_` or
   `monitor_`; the write-gate test treats every other tool as mutating and
   fails if it runs while writes are off.

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

TLS verification is on by default. `MIKROTIK_CA_FILE` pins the router's
certificate; `MIKROTIK_TLS_INSECURE=true` and `MIKROTIK_SCHEME=http` are
explicit opt-ins that warn at startup. Don't add a code path that turns
verification off or picks http on its own. `src/config.rs` owns all env
parsing, and boolean variables go through `parse_bool_env`.

When iterating, run `bacon` in a second terminal. The server detects binary
mtime changes and exits, and the MCP client auto-restarts it. See `bacon.toml`.

## Before committing

All hooks must pass. Run them with [`prek`][prek], a Rust reimplementation of
pre-commit. **Do not use Python `pre-commit` or `uvx pre-commit`.**

```sh
prek run --all-files
```

Hooks: trailing-whitespace, end-of-file, check-toml/yaml, merge-conflict,
large-files, typos, editorconfig-checker, markdownlint-cli2, `cargo fmt
--check`, `cargo clippy --all-targets -- -D warnings`.

Clippy is set to deny warnings. Fix each lint; don't silence it with `#[allow]`.
Spell checking is [`typos`][typos] (Rust), which flags only *known misspellings*,
so RouterOS menu paths, crate names and domain vocabulary need no dictionary
entries. For a false positive, add the word to `_typos.toml` under
`[default.extend-words]` with a comment saying why; don't add a blanket exclusion.

[prek]: https://github.com/j178/prek
[typos]: https://github.com/crate-ci/typos

## Branch naming

**`main` is the only long-lived branch.** Branch off it, PR back into it.
There is no `develop` branch. It was merged into `main` in PR #10 and deleted,
and every PR since has targeted `main` directly.

- `main`: the trunk. Released code, tagged versions, and the base for all
  work. Merges arrive by PR; release PRs come from release-please.
- `feat/<short-name>`: new features. Example: `feat/firewall-rules`.
- `fix/<short-name>` or `bugfix/<short-name>`: bug fixes.
- `docs/<short-name>`, `chore/<short-name>`, `refactor/<short-name>`:
  documentation, chores and refactors.

`feature/<short-name>` also appears in the history and still works; `feat/` is
what recent branches use. Pick one and stay consistent within a branch.

Use lowercase kebab-case slugs, such as `feat/dhcp-lease-tools`. Keep names
short and name the topic; branch names carry no ticket numbers.

## Commit messages

Conventional commits. The version is cut by release-please from these prefixes:

- `feat: …` → minor bump
- `fix: …`, `perf: …` → patch bump
- `feat!: …` or `BREAKING CHANGE:` footer → major bump
- `chore: …`, `ci: …`, `test: …`, `docs: …`, `refactor: …` → no version bump
  (`release-please-config.json` sets which of them appear in the changelog)

Keep one logical change per commit. Don't squash unrelated edits together.

## Releases

Don't bump the version manually. release-please opens a PR that bumps
`Cargo.toml`, `Cargo.lock`, `.release-please-manifest.json`, and the install
tag inside `README.md` (between the `x-release-please-start-version` HTML
markers). Merging that PR cuts the tag and triggers binary builds.

## CI

`.github/workflows/ci.yml` runs prek, check, fmt, clippy, test, and doc on
every PR into `main`, on pushes to `main`, and on pushes to `feat/**`,
`feature/**`, `fix/**`, `bugfix/**`, `docs/**`, `chore/**` and `refactor/**`.
All jobs must pass.

A branch whose prefix is not in that list gets CI from its **pull request**
only, with no push-triggered run. If you add a new prefix
convention, add it to `ci.yml` too.

`.github/workflows/release-please.yml` watches pushes to `main` and maintains
the release PR. `.github/workflows/release.yml` fires only on a **published
release** and skips pre-releases.

`.github/workflows/sbom-scan.yml` generates the SBOM and scans it for CVEs. It
runs weekly on a schedule (gating) and from `release.yml` at release time
(informational). See **Security scanning** below.

## Security scanning

`sbom-scan.yml` runs Syft (SBOM: CycloneDX + SPDX) then Grype (CVE scan) over
the crate's dependency tree.

| Trigger | Mode | Effect |
| --- | --- | --- |
| Weekly schedule | **gating** | Fails on Critical/High with a fix |
| Release published | informational | Attaches SBOMs to the release |
| `workflow_dispatch` | your choice | Fails only if you select `gating` |

A release is never gated: by the time `release.yml` runs the version is already
published, so failing there would leave a half-shipped release. Critical/High
findings with **no** fix available are logged and do not fail the run, because
there is no fix to act on. Medium gets a `::warning::`; Low is logged only.

Two conventions in that workflow differ from the rest of the repo, on purpose:

- **Actions are pinned by commit SHA.** Tags are mutable, and a workflow whose
  job is supply-chain assurance must not depend on a mutable reference.
  Everything else in `.github/workflows/` pins by major tag (`@v7`).
- **Syft and Grype are installed from checksum-verified release tarballs.**
  Don't replace either convention with a wrapper Action, a tag pin or
  `curl | sh`.

When bumping the pinned `SYFT_VERSION` / `GRYPE_VERSION`, keep the checksum
verification step intact.

**The SBOM is lockfile-scoped, so it is a superset of what ships.**
`syft scan dir:.` reads `Cargo.lock`, and Cargo records optional dependencies of
dependencies even when the feature that would pull them in is disabled. Before
treating a finding as real exposure, check whether the crate is built at all:

```sh
cargo tree -i <crate>                 # "nothing to print" => not in the build graph
cargo tree -i <crate> --target all    # check other platforms too
```

A crate absent from the build graph is a lockfile hygiene issue with no
exposure. Fix it if the fix is cheap (`cargo update -p <crate>`, then
`cargo test --locked`), and don't treat it as an incident. The first finding
this scanning produced was that case: a High advisory in a QUIC crate
reachable only through a `reqwest` feature this crate does not enable.

If unbuilt optional dependencies get noisy, build with `cargo auditable` and
scan the binary, which embeds the real dependency set. This is not done today
because it adds a build-time dependency to solve a problem this crate does not
have.

The scan covers the crate dependency tree and the GitHub Actions pinned in
`.github/workflows/` (Syft catalogs those too). It does not cover the Rust
toolchain, the runner image, or the RouterOS devices this server talks to.

## Tool descriptions

Tool descriptions (the `description = "…"` string in each `#[tool(…)]` attribute)
must stay close to the language used in the official MikroTik documentation. Use
the same field names, menu paths, and terminology that RouterOS uses. Don't invent
synonyms or over-explain RouterOS internals.

When adding or updating a tool, look up the relevant wiki page first and mirror
its terminology. Key references:

| Area | Wiki page |
|------|-----------|
| REST API general | <https://help.mikrotik.com/docs/spaces/ROS/pages/47579229/REST+API> |
| IP neighbors (CDP/LLDP/MNDP) | <https://help.mikrotik.com/docs/spaces/ROS/pages/8323118/IP+Neighbors> |
| IP addresses | <https://help.mikrotik.com/docs/spaces/ROS/pages/328088/IP+Address> |
| DHCP server | <https://help.mikrotik.com/docs/spaces/ROS/pages/24805500/DHCP> |
| Firewall filter | <https://help.mikrotik.com/docs/spaces/ROS/pages/48660574/Filter> |
| Interfaces | <https://help.mikrotik.com/docs/spaces/ROS/pages/328155/Ethernet> |
| Wireless (legacy) | <https://help.mikrotik.com/docs/spaces/ROS/pages/1409044/Wireless> |
| CAPsMAN | <https://help.mikrotik.com/docs/spaces/ROS/pages/1409149/CAPsMAN> |
| Routes | <https://help.mikrotik.com/docs/spaces/ROS/pages/328084/IP+Routing> |
| DNS | <https://help.mikrotik.com/docs/spaces/ROS/pages/24805404/DNS> |

Links were last verified 2026-05-26. Re-check them periodically, because
MikroTik moves pages without redirects. Run `curl -sI <url> | grep -i location`
to spot moved pages, or search [help.mikrotik.com](https://help.mikrotik.com)
by topic.

## The write gate is a convenience guard

Mutating tools return `INVALID_REQUEST` unless `MIKROTIK_ALLOW_WRITES=true`
is set. Anyone who can set the server's environment can open the gate. Enforce
read-only access at the RouterOS user level (`policy=read,api,rest-api`).

When a write fails because the gate is closed, tell the user to set
`MIKROTIK_ALLOW_WRITES=true` in the MCP config env block. Don't retry or
work around it.

Read current state before proposing any write. Confirm with the user before
`remove_*` calls. Ask for explicit confirmation before every `restore_backup`
call, because the device reboots. The tool refuses to run without
`confirm: true`; set it only after the user has said yes to that restore.

## Things to avoid

- Don't write to stdout from the server process. Stdout is the MCP protocol
  channel; use `tracing` (stderr) for diagnostics.
- Don't add dependencies casually; this is a small surface. Justify in the
  commit message.
- Don't touch `Cargo.lock` by hand; let cargo manage it.
- Don't disable lints to make CI pass.
- Don't add AI attribution to commits. No `Co-Authored-By` trailers for AI
  tools. Commits are authored by the human who reviews and accepts the change.
