# Torii ⛩️ — Roadmap

Living document. Sections are prioritized; within a section items are ordered by impact.
"Released" = on crates.io. "In progress" = work started or imminent. "Future" = direction, not commitment.

---

## Released

### v0.6.0 — Pure-Rust transports (May 2026)
- Custom HTTPS transport over `reqwest` + `rustls`, replacing libgit2's libcurl path
- Custom SSH transport over `russh` + `aws-lc-rs`, replacing libgit2's libssh2 path
- libgit2 vendored without HTTPS/SSH support (`GIT_HTTPS=0 GIT_SSH=0`)
- HTTPS auth via env vars: `GITHUB_TOKEN`, `GITLAB_TOKEN`, `CODEBERG_TOKEN`, `BITBUCKET_TOKEN`, `GITEA_TOKEN`, `FORGEJO_TOKEN`, `SOURCEHUT_TOKEN`, `TORII_HTTPS_TOKEN`
- SSH auth chain: ssh-agent → `~/.ssh/id_ed25519` → `~/.ssh/id_rsa`
- SSH host verification via `~/.ssh/known_hosts` (hashed entries + `[host]:port`), TOFU prompt on tty, `TORII_SSH_STRICT=1` to force strict
- Actionable HTTPS errors (401 / 403 / 404)
- Build deps reduced to a C compiler — no perl, no openssl-dev, no libssh2-dev, no pkg-config
- Fix: silent push rejections — `remote.push()` returns Ok even when the server rejects; now collected via `push_update_reference` and reported as `push rejected by remote: <ref> → <reason>`. Pre-existed the rewrite, affected 0.5.0 too.

### v0.5.0 — Declarative gates + machine-private overlay (April 2026)
- `.toriignore.local` — machine-private overlay, auto-gitignored
- `torii ignore add | secret | list`
- `[secrets]`, `[size]`, `[hooks]` sections in `.toriignore`
- Update banner in TUI when newer crates.io version available; CLI update notifier
- Command surface tightened: promoted `blame`/`scan`/`cherry-pick`, demoted `ls`/`unstage`/`repo`
- Fixed: `pull` branch handling, `remote link`, `unstage`, `rebase --root`, `amend` after history rewrite, `branch --orphan`, `save` flag combinations

### v0.4.0 and earlier — Core git surface
- `save / sync / status / log / diff / branch / clone / cherry-pick / blame`
- Rebase: `--continue / --abort / --skip`, interactive, `--todo-file`, `--root`
- Snapshots: `create / list / restore / delete / stash / unstash / undo`, auto-snapshot
- History: `rewrite / clean / verify-remote / reflog / remove-file`
- Tags: `create / list / delete / push / show / release` (auto-bump from conventional commits, `--bump`, `--dry-run`)
- Scanner: staged + history (`--history`), pre-save hook, JWT/AWS/GH/GL/Stripe/Twilio/PEM/DB/API patterns
- Mirrors: GitHub, GitLab, Codeberg, Bitbucket, Gitea, Forgejo, Sourcehut, SourceForge, custom servers; primary/replica model; autofetch
- Remote management: `remote create / delete / visibility / configure / info / list`
- Workspace: batch operations across multiple repos
- Config: global + local, `set / get / list / edit / reset`
- Custom workflow aliases: `custom add / list / run / remove`
- TUI: PR/MR view, commit amend from TUI, branch/tag search, background loading

---

## In progress / next

### v0.6.1 — Static binary (branch `feat/static-binary`, ready)
- `static` Cargo feature → vendored zlib via `libz-sys/static`
- Build target `x86_64-unknown-linux-musl` produces a binary with **zero runtime libs** (runs on Alpine, scratch, busybox, any glibc/musl mix)
- GitLab CI gains `build-linux-x86_64-musl` job
- Verified locally: 18MB statically-linked binary, clones github.com over HTTPS

### Validation and polish
- Push validation against Bitbucket, Gitea, Forgejo, Sourcehut (transports expected to work — same Smart HTTP/SSH protocol — but not individually verified)
- SSH passphrase prompt for encrypted disk keys (currently only unencrypted keys + agent)
- `~/.ssh/config` parsing (HostName/User/IdentityFile/Port aliases)
- Re-enable GitHub Actions for auto-publish (currently manual `cargo publish` from local)
- Integration tests against a local git server (so transport regressions are caught in CI, not by users)

### Platform API completion
- Codeberg — repo create/delete/visibility/info via Forgejo API (HTTPS push works today; remote create still falls back to manual)
- Gitea — remote management API
- Forgejo — remote management API
- Bitbucket — remote management API hardening

### Monetization (gitorii.com)
- Paddle integration (schema ready, integration pending)
- Indie (free) / Scale (20€/mo) / Teams (30€/mo) / Seed (10€/mo) / Enterprise (custom)
- Target: 17–20k€ ARR

---

## Future

### Gate — CI/CD transpiler (separate repo: `gate`)
A DSL that compiles to GitHub Actions / GitLab CI / CircleCI / Azure / Bitbucket Pipelines YAML. Not a runner — pure transpiler. Open-source docs, no paywall. Integrates with torii so `torii sync` can validate CI is in sync with the source DSL.

### AI generation
- AI-assisted commit messages from staged diff
- Conventional-commits validator + suggester
- Natural-language "what changed in the last week" summaries

### TUI / GUI roadmap
- TUI: PR/MR creation flow, conflict resolution UI, snapshot diff browser
- Tauri GUI long-term — natural language interface with embedded console fallback

### New VCS
Long horizon. Torii is currently a git wrapper. The long-term goal is a VCS designed around human workflows rather than git's technical model. Torii would provide the migration path.

### Scanner improvements
- `scan --fix` — auto-remove detected secrets from staged files
- Custom regex patterns via `.toriignore` `[secrets]` (already partially shipped in 0.5.0; expand)
- Pre-push scan, not just pre-save
- Org-wide policy file shared across repos

### Interactive staging
- `torii save --patch` — stage hunks interactively (`git add -p` equivalent)

### Log improvements
- `torii log --graph` — visual branch graph
- `torii log -S <string>` — pickaxe search

### Snapshot improvements
- Compression for old entries
- Remote snapshot backup (private bucket, opt-in)
- Diff between two snapshots

### Migration paths
- `torii migrate` — guided import from a `git` workflow (rename familiar commands, set up `.toriignore` from existing `.gitignore`, etc.)
- Long-term: migration paths to the new VCS once it exists

---

## Considered, not planned

These come up but are deliberately out of scope.

- **gitoxide migration.** Evaluated May 2026: push incomplete, rebase API unstable, no production users. Revisit late 2026 / early 2027 once stabilized.
- **AWS CodeCommit support.** AWS deprecated new repos in 2024.
- **Mercurial / Bazaar / Pijul / Fossil interop.** git-only for the foreseeable future.
- **Web UI hosted on gitorii.com that browses your repos.** That's GitHub/GitLab/Codeberg's job. gitorii.com stays a marketing + docs + billing site.

---

*Last updated: 2026-05-02 (after v0.6.0 ship)*
