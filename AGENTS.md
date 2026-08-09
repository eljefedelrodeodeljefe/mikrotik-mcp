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
2. Add a pure function to `src/tools/{domain}.rs` — takes `&RouterosClient`
   and any params, returns `anyhow::Result<Value>`. Add a wiremock test.
3. Add a short `#[tool(description = "…")]` stub in the matching `impl`
   block in `src/server.rs`. Call `self.guard_write()?` first for any
   mutating tool.

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

## Branch naming

**`main` is the only long-lived branch.** Branch off it, PR back into it.
There is no `develop` — it was merged into `main` in PR #10 and deleted, and
every PR since has targeted `main` directly.

- `main` — the trunk. Released code, tagged versions, and the base for all
  work. Merges arrive by PR; release PRs come from release-please.
- `feat/<short-name>` — new features. Example: `feat/firewall-rules`.
- `fix/<short-name>` or `bugfix/<short-name>` — bug fixes.
- `docs/<short-name>`, `chore/<short-name>`, `refactor/<short-name>` — as the
  name suggests.

`feature/<short-name>` also appears in the history and still works; `feat/` is
what recent branches use. Pick one and stay consistent within a branch.

Use kebab-case slugs (`feat/dhcp-lease-tools`, not `feat/DHCP_Lease_Tools`).
Keep names short and topic-focused, not ticket-numbered (we don't use issue
trackers as filenames here).

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
every PR into `main`, on pushes to `main`, and on pushes to `feat/**`,
`feature/**`, `fix/**`, `bugfix/**`, `docs/**`, `chore/**` and `refactor/**`.
All jobs must pass.

A branch prefix that is not in that list still gets CI from its **pull
request** — only the push-triggered run is skipped. If you add a new prefix
convention, add it to `ci.yml` too.

`.github/workflows/release-please.yml` watches pushes to `main` and maintains
the release PR. `.github/workflows/release.yml` is separate again: it fires on
a **published release**, not on pushes, and skips pre-releases.

`.github/workflows/sbom-scan.yml` generates the SBOM and scans it for CVEs —
weekly on a schedule (gating), and from `release.yml` at release time
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
findings with **no** fix available are logged, not gated — there is nothing
actionable to gate on. Medium gets a `::warning::`; Low is logged only.

Two conventions in that workflow differ from the rest of the repo, on purpose:

- **Actions are pinned by commit SHA, not by major tag.** Tags are mutable; a
  workflow whose job is supply-chain assurance must not depend on a mutable
  reference. Everything else in `.github/workflows/` pins by tag (`@v7`).
- **Syft and Grype are installed from checksum-verified release tarballs**, not
  from a wrapper Action or `curl | sh`. Don't "simplify" either of these.

When bumping the pinned `SYFT_VERSION` / `GRYPE_VERSION`, keep the checksum
verification step intact.

**The SBOM is lockfile-scoped, so it is a superset of what actually ships.**
`syft scan dir:.` reads `Cargo.lock`, and Cargo records optional dependencies of
dependencies even when the feature that would pull them in is disabled. Before
treating a finding as real exposure, check whether the crate is built at all:

```sh
cargo tree -i <crate>                 # "nothing to print" => not in the build graph
cargo tree -i <crate> --target all    # check other platforms too
```

A crate absent from the build graph is lockfile hygiene, not an exposure — fix it
if the fix is cheap (`cargo update -p <crate>`, then `cargo test --locked`), but
don't treat it as an incident. The first finding this scanning produced was
exactly that case: a High advisory in a QUIC crate reachable only through a
`reqwest` feature this crate does not enable.

If unbuilt optional dependencies ever get noisy, the fix is to build with
`cargo auditable` and scan the binary, which embeds the real dependency set.
Deliberately not done today — it adds a build-time dependency to solve a problem
this crate does not have.

The scan covers the crate dependency tree and the GitHub Actions pinned in
`.github/workflows/` (Syft catalogs those too). It does not cover the Rust
toolchain, the runner image, or the RouterOS devices this server talks to.

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
| Firewall filter | <https://help.mikrotik.com/docs/spaces/ROS/pages/48660574/Filter> |
| Interfaces | <https://help.mikrotik.com/docs/spaces/ROS/pages/328155/Ethernet> |
| Wireless (legacy) | <https://help.mikrotik.com/docs/spaces/ROS/pages/1409044/Wireless> |
| CAPsMAN | <https://help.mikrotik.com/docs/spaces/ROS/pages/1409149/CAPsMAN> |
| Routes | <https://help.mikrotik.com/docs/spaces/ROS/pages/328084/IP+Routing> |
| DNS | <https://help.mikrotik.com/docs/spaces/ROS/pages/24805404/DNS> |

Links were last verified 2026-05-26. Re-check periodically — MikroTik moves pages
without redirects. Run `curl -sI <url> | grep -i location` to spot moved pages,
or search [help.mikrotik.com](https://help.mikrotik.com) by topic.

## Write gate

Mutating tools return `INVALID_REQUEST` unless `MIKROTIK_ALLOW_WRITES=true`
is set. This is a convenience guard, not a security boundary — real
enforcement belongs at the RouterOS user level (`policy=read,api,rest-api`).

When a write fails because the gate is closed, tell the user to set
`MIKROTIK_ALLOW_WRITES=true` in the MCP config env block. Don't retry or
work around it.

Read current state before proposing any write. Confirm with the user before
`remove_*` calls. Treat `restore_backup` as requiring explicit confirmation
every time — the device reboots.

## Things to avoid

- Don't write to stdout from the server process — stdout is the MCP protocol
  channel. Use `tracing` (stderr) for diagnostics.
- Don't add dependencies casually; this is a small surface. Justify in the
  commit message.
- Don't touch `Cargo.lock` by hand; let cargo manage it.
- Don't disable lints to make CI pass.
- Don't add AI attribution to commits. No `Co-Authored-By` trailers for AI
  tools — commits are authored by the human who reviews and accepts the change.
