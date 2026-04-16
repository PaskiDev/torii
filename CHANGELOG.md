# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.6] - 2026-04-16

### Added
- `torii sync` now automatically pushes to all configured replica mirrors after syncing with origin — no need to run `torii mirror sync` manually

### Fixed
- Removed unused `mut` on `index` variable in rebase loop
- Removed unused `repo_path` variable in `show` command

## [0.1.5] - 2025-04-16

### Fixed
- Full Windows and macOS native compatibility — removed all `HOME` env var hardcoding
- Replaced all `Command::new("git")` subprocesses with native `git2` API calls
- SSH credential resolution now uses `dirs::home_dir()` for cross-platform paths
- Config dir now uses platform-native path via `dirs::config_dir()` (Linux XDG, macOS `~/Library`, Windows `%APPDATA%`)
- `torii snapshot restore` uses git2 hard reset instead of subprocess
- `torii snapshot stash/unstash` uses git2 index and reset instead of subprocess
- `torii history reflog` uses git2 reflog API
- `torii history revert/reset/merge/rebase` fully ported to git2
- Tags pushed via git2 enumeration instead of `git push --tags` subprocess
- `OpenSSL` vendored in `git2` dependency for Windows native builds

### Changed
- `git2` dependency updated to include `vendored-openssl` feature for Windows support

## [0.1.4] - 2025-03-15

### Changed
- Renamed `master`/`slave` mirror terminology to `primary`/`replica` across all commands and output
  - `torii mirror add-master` → `torii mirror add-primary`
  - `torii mirror add-slave` → `torii mirror add-replica`

### Fixed
- Platform-native config path for token storage (was hardcoded to Linux `~/.config`)

## [0.1.3] - 2025-03-01

### Added
- Platform shorthand syntax for `torii clone`: `torii clone github user/repo`
- `torii ls [PATH]` — list tracked files in the index
- `torii show [OBJECT]` — show commit, tag or file details
- `torii history` subcommand group consolidating 7 previously top-level commands

### Fixed
- `torii history remove-file` now works on directories (`-r` flag)
- Wildcard matching in `.toriignore`
- Removed dead `integrate` code

## [0.1.2] - 2025-02-15

### Changed
- Collapsed 7 top-level history-related commands into `torii history` subcommands for a cleaner CLI surface

### Fixed
- Repo URL in Cargo.toml

## [0.1.1] - 2025-02-01

### Added
- `torii history remove-file` — permanently erase a file from the entire git history

### Fixed
- Scanner now detects sensitive filenames (`.env`, `*.pem`, `id_rsa`, etc.)
- Scanner extended with Spanish-language placeholder detection
- Reduced false positives in sensitive data scanner
- Mirror sync now pushes tags alongside branch refs
- GitHub remote creation uses REST API instead of shelling out to `gh` CLI
- Support for root commit in empty repositories
- `.toriignore` wildcard matching and ref handling
- Explicit SSH key used for mirror sync

### Changed
- `.gitignore` renamed to `.toriignore` — Torii manages its own ignore file
- Custom workflows moved to `torii-premium`
- Entire `.torii/` directory excluded from tracking
- Crate renamed to `gitorii` for crates.io publication

## [0.1.0] - 2025-01-15

### Added
- Core git operations: `torii init`, `torii clone`, `torii save`, `torii sync`, `torii status`, `torii diff`, `torii log`
- Branch management: `torii branch`, `torii switch`, `torii merge`
- Snapshot system: `torii snapshot create/list/restore/stash/unstash`
- Multi-platform mirror sync: `torii mirror add-primary/add-replica/list/sync`
- Remote repository management: `torii remote create/delete/list` (GitHub, GitLab, Gitea, Forgejo, Codeberg, Sourcehut, SourceForge)
- Tag management and auto-versioning: `torii tag`
- Built-in sensitive data scanner (pre-save and history scan)
- History rewriting: `torii history rewrite/rebase/cherry-pick/reflog/blame`
- Custom config system: global (`~/.config/torii`) and local (`.torii/`)
- `.toriignore` support synced to `.git/info/exclude`
- SSH authentication helper
- Duration parsing utilities (`10m`, `2h`, `1d`)
- Multi-platform URL generation (SSH and HTTPS)
- Autofetch configuration for mirrors

[Unreleased]: https://gitlab.com/paskidev/torii/-/compare/v0.1.5...HEAD
[0.1.5]: https://gitlab.com/paskidev/torii/-/compare/v0.1.4...v0.1.5
[0.1.4]: https://gitlab.com/paskidev/torii/-/compare/v0.1.3...v0.1.4
[0.1.3]: https://gitlab.com/paskidev/torii/-/compare/v0.1.2...v0.1.3
[0.1.2]: https://gitlab.com/paskidev/torii/-/compare/v0.1.1...v0.1.2
[0.1.1]: https://gitlab.com/paskidev/torii/-/compare/v0.1.0...v0.1.1
[0.1.0]: https://gitlab.com/paskidev/torii/-/releases/tag/v0.1.0
