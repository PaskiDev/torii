# Torii ⛩️ — Roadmap

## Released — v0.1.0

### Core git operations
- [x] `torii save` — commit with amend, revert, reset, selective file staging
- [x] `torii sync` — push/pull/fetch, force push, branch integration, auto tag push
- [x] `torii status` — repository status with suggestions
- [x] `torii log` — history with --author, --since, --until, --grep, --stat filters
- [x] `torii diff` — unstaged, staged, last commit
- [x] `torii branch` — list (local + remote), create, switch, delete, rename
- [x] `torii clone` — with platform shortcuts (github, gitlab, codeberg…)
- [x] `torii cherry-pick` — apply commits with --continue / --abort
- [x] `torii blame` — line-by-line with range support

### Rebase
- [x] `torii rebase <target>` — rebase onto branch or commit
- [x] `torii rebase -i <target>` — interactive rebase (opens editor)
- [x] `torii rebase --todo-file <file>` — non-interactive with pre-written todo
- [x] `torii rebase --continue / --abort / --skip`

### Snapshots
- [x] `torii snapshot create / list / restore / delete`
- [x] `torii snapshot stash / unstash` — including untracked files (`-u`)
- [x] `torii snapshot undo` / `torii undo`
- [x] Auto-snapshot configuration

### History
- [x] `torii history rewrite` — rewrite commit dates across full history
- [x] `torii history clean` — gc + reflog expire
- [x] `torii history verify-remote`
- [x] `torii history reflog` — explore HEAD movement history

### Tags & releases
- [x] `torii tag create / list / delete / push / show`
- [x] `torii tag release` — auto-bump from conventional commits since last tag
- [x] `torii tag release --bump major|minor|patch` — manual override
- [x] `torii tag release --dry-run`

### Security scanner
- [x] `torii scan` — scan staged files for secrets before committing
- [x] `torii scan --history` — scan entire git history
- [x] Pre-save hook — warns and asks confirmation on detection
- [x] Detects: JWT, AWS keys, GitHub/GitLab tokens, Stripe, Twilio, PEM keys, DB connection strings, generic API keys
- [x] Whitelists `.example`, `.sample`, `.template` files automatically

### Mirrors
- [x] `torii mirror add-master / add-slave / list / sync / remove / set-master`
- [x] `torii mirror autofetch` — automatic background sync
- [x] Platforms: GitHub, GitLab, Codeberg, Bitbucket, Gitea, Forgejo, SourceHut, SourceForge, custom servers
- [x] SSH / HTTPS auto-detection

### Remote repository management
- [x] `torii remote create / delete / visibility / configure / info / list`
- [x] `torii repo` — batch operations across multiple platforms simultaneously

### Config & utilities
- [x] `torii config set / get / list / edit / reset` — global + local
- [x] `torii custom add / list / run / remove` — custom workflow aliases
- [x] `torii ssh-check`
- [x] `.toriignore` support

---

## In progress / Next

### Platform API completion
- [ ] Gitea — remote management API (stubbed, returns not implemented)
- [ ] Forgejo — remote management API
- [ ] Codeberg — remote management API

### Scanner improvements
- [ ] `torii scan --fix` — auto-remove detected secrets from staged files
- [ ] Custom pattern rules via `.toriignore` or config
- [ ] Pre-push scan (not just pre-save)

### Snapshot improvements
- [ ] Snapshot compression for old entries
- [ ] Remote snapshot backup
- [ ] Diff between two snapshots

### Interactive staging
- [ ] `torii save --patch` — stage hunks interactively (like `git add -p`)

### Log improvements
- [ ] `torii log --graph` — visual branch graph
- [ ] `torii log -S <string>` — pickaxe search

---

## Future

### GUI (Tauri)
Natural language interface with embedded console for advanced users. No commands required for common operations — the focus is on intent, not syntax.

### New VCS
Torii is currently a Git wrapper. The long-term goal is a new version control system built from scratch around human workflows rather than technical implementation details. Torii will provide a migration path from Git to the new VCS.

### CI/CD portable configuration
- Generate CI/CD configs for multiple platforms from a single source
- Validate and sync across GitHub Actions, GitLab CI, and others

---

*Last updated: April 2026*
