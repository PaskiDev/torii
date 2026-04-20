# Gitorii (torii) — Command Reference

> Install: `cargo install gitorii`  
> Binary: `torii`  
> Version: 0.1.16

---

## Quick Start

```bash
torii init                        # New repo
torii save -am "feat: add login"  # Stage all + commit
torii sync                        # Pull + push
torii status                      # What changed
```

---

## `torii init`

Initialize a new git repository.

```bash
torii init                          # Current directory
torii init --path ~/projects/myrepo # Specific path
```

---

## `torii save`

Simplified commit. Replaces `git add` + `git commit`.

```bash
torii save -m "fix: null check"              # Commit staged changes
torii save -am "feat: add login"             # Stage all + commit
torii save src/auth.rs -m "fix: token"       # Stage specific file + commit
torii save --amend -m "fix: typo"            # Amend last commit
torii save --revert abc1234 -m "revert"      # Revert a commit
torii save --reset HEAD~1 --reset-mode soft  # Undo last commit, keep changes staged
torii save --reset HEAD~1 --reset-mode mixed # Undo last commit, unstage changes (default)
torii save --reset HEAD~1 --reset-mode hard  # Undo last commit, discard changes
```

| Flag | Description |
|------|-------------|
| `-m` | Commit message (required) |
| `-a` / `--all` | Stage all changes before committing |
| `--amend` | Amend previous commit |
| `--revert <hash>` | Revert a specific commit |
| `--reset <hash>` | Reset to a specific commit |
| `--reset-mode` | `soft` / `mixed` / `hard` (default: mixed) |

---

## `torii sync`

Pull + push in one command. Also integrates branches.

```bash
torii sync                   # Pull then push
torii sync --pull            # Pull only
torii sync --push            # Push only
torii sync --force           # Force push (rewrites remote history)
torii sync --fetch           # Fetch remote refs without merging

# Branch integration
torii sync main              # Integrate main into current branch (smart)
torii sync main --merge      # Force merge strategy
torii sync main --rebase     # Force rebase strategy
torii sync main --preview    # Preview without executing
```

---

## `torii status`

Show working tree status: staged, unstaged, and untracked files.

```bash
torii status
```

---

## `torii log`

Show commit history.

```bash
torii log                          # Last 10 commits
torii log -n 50                    # Last 50 commits
torii log --oneline                # One line per commit
torii log --oneline --graph        # Compact graph view
torii log --author "Alice"         # Filter by author
torii log --since 2024-01-01       # Commits after date
torii log --until 2024-12-31       # Commits before date
torii log --grep "feat"            # Filter by message pattern
torii log --stat                   # File change stats per commit
```

---

## `torii diff`

Show changes.

```bash
torii diff            # Unstaged changes
torii diff --staged   # Staged changes (ready to commit)
torii diff --last     # Changes in last commit
```

---

## `torii branch`

Manage branches.

```bash
torii branch                      # List local branches
torii branch --all                # List local + remote branches
torii branch feature/auth -c      # Create and switch to branch
torii branch main                 # Switch to existing branch
torii branch -d feature/auth      # Delete branch
torii branch --rename new-name    # Rename current branch
```

---

## `torii clone`

Clone a repository. Supports platform shorthands and full URLs.

```bash
torii clone github user/repo                    # Auto SSH/HTTPS
torii clone gitlab user/repo                    # GitLab
torii clone github user/repo --protocol https   # Force HTTPS
torii clone github user/repo -d my-dir          # Custom directory
torii clone https://github.com/user/repo.git    # Full URL
torii clone git@github.com:user/repo.git        # SSH URL
```

**Platforms:** `github`, `gitlab`, `codeberg`, `bitbucket`, `gitea`, `forgejo`

Protocol auto-detected: SSH if keys present, HTTPS otherwise.  
Override: `torii config set mirror.default_protocol https`

---

## `torii tag`

Manage tags and releases.

```bash
torii tag list                           # List all tags
torii tag create v1.2.0 -m "Release"    # Create annotated tag
torii tag delete v1.0.0                 # Delete tag
torii tag push v1.2.0                   # Push specific tag
torii tag push                          # Push all tags
torii tag show v1.2.0                   # Show tag details

# Auto-release from conventional commits
torii tag release                        # Auto-bump version
torii tag release --bump minor           # Force minor bump
torii tag release --dry-run              # Preview without creating
```

**Auto-bump rules (Conventional Commits):**

| Commit type | Version bump |
|-------------|-------------|
| `feat:` | minor (0.1.0 → 0.2.0) |
| `fix:` / `perf:` | patch (0.1.0 → 0.1.1) |
| `feat!:` | major (0.1.0 → 1.0.0) |

---

## `torii snapshot`

Save and restore work-in-progress states. Unlike git stash, snapshots are named, persistent, and don't affect your working tree until explicitly restored.

```bash
torii snapshot create -n "before-refactor"  # Named snapshot
torii snapshot list                          # List all snapshots
torii snapshot restore <id>                  # Restore a snapshot
torii snapshot delete <id>                   # Delete a snapshot

# Stash (quick save/restore)
torii snapshot stash                         # Stash current work
torii snapshot stash -u                      # Include untracked files
torii snapshot unstash                       # Restore latest stash
torii snapshot unstash <id> --keep           # Restore but keep stash

# Undo
torii snapshot undo                          # Undo last operation

# Auto-snapshot config
torii snapshot config                        # Show auto-snapshot settings
```

---

## `torii mirror`

Mirror your repo across multiple platforms simultaneously.

```bash
# Setup
torii mirror add-primary gitlab user paskidev myrepo   # Set primary (source of truth)
torii mirror add-replica github user paskidev myrepo   # Add replica mirror

# Sync
torii mirror sync                   # Push to all replicas
torii mirror sync --force           # Force push to all replicas

# Manage
torii mirror list                   # List configured mirrors
torii mirror set-primary github     # Change primary
torii mirror remove github user     # Remove a mirror

# Auto-fetch
torii mirror autofetch --enable --interval 30m   # Auto-fetch every 30 min
torii mirror autofetch --disable                  # Disable
torii mirror autofetch --status                   # Show status
```

**Platforms:** `github`, `gitlab`, `codeberg`, `bitbucket`, `gitea`, `forgejo`

---

## `torii ls`

List all tracked files in the index.

```bash
torii ls           # All tracked files
torii ls src/      # Files under src/
```

---

## `torii show`

Show details of a commit, tag, or file.

```bash
torii show                        # HEAD commit with diff
torii show abc1234                # Specific commit
torii show v1.0.0                 # Tag details
torii show src/main.rs --blame    # Line-by-line change history
torii show src/main.rs --blame -L 10,20   # Blame specific range
```

---

## `torii ssh-check`

Verify SSH key configuration and print setup instructions if needed.

```bash
torii ssh-check
```

---

## `torii history`

Advanced history operations.

```bash
# Rebase
torii history rebase main              # Rebase onto main
torii history rebase -i HEAD~5         # Interactive rebase last 5 commits
torii history rebase --continue        # Continue after resolving conflicts
torii history rebase --abort           # Abort rebase
torii history rebase --skip            # Skip current patch

# Cherry-pick
torii history cherry-pick abc1234      # Apply commit to current branch

# Blame
torii history blame src/main.rs        # Line-by-line change history
torii history blame src/main.rs -L 10,20  # Specific line range

# Secret scanning
torii history scan                     # Scan staged files for secrets
torii history scan --history           # Scan entire git history

# History rewrite
torii history rewrite "2024-01-01 00:00" "2024-12-31 23:59"  # Rewrite commit dates
torii history remove-file secrets.txt  # Purge file from all commits
torii history clean                    # GC + expire reflog

# Other
torii history reflog                   # HEAD movement history
torii history verify-remote            # Compare local vs remote HEAD
```

### Secret scanner patterns

Detects automatically:
- Private keys (PEM)
- JWT tokens
- AWS access/secret keys
- GitHub / GitLab tokens (`ghp_`, `glpat-`, etc.)
- Generic API keys / passwords
- Database connection strings with credentials
- Stripe keys (`sk_live_`, `pk_live_`)
- Twilio / SendGrid / Brevo keys

Skips: example files (`.env.example`), i18n files, binary files, lock files.

---

## `torii config`

Manage global and local configuration.

```bash
torii config list                               # All config values
torii config list --local                       # Local repo config
torii config get user.name                      # Get a value
torii config set user.name "Alice"              # Set global value
torii config set user.email "a@b.com" --local  # Set local value
torii config edit                               # Open in editor
torii config reset                              # Reset to defaults
```

**Available keys:**

| Key | Description |
|-----|-------------|
| `user.name` | Git author name |
| `user.email` | Git author email |
| `user.editor` | Preferred editor |
| `auth.github_token` | GitHub personal access token |
| `auth.gitlab_token` | GitLab personal access token |
| `auth.gitea_token` | Gitea token |
| `auth.forgejo_token` | Forgejo token |
| `auth.codeberg_token` | Codeberg token |
| `git.default_branch` | Default branch name |
| `git.sign_commits` | GPG sign commits |
| `git.pull_rebase` | Rebase on pull |
| `mirror.default_protocol` | `ssh` or `https` |
| `mirror.autofetch_enabled` | Auto-fetch from mirrors |
| `snapshot.auto_enabled` | Auto-snapshots |
| `snapshot.auto_interval_minutes` | Auto-snapshot interval |
| `ui.colors` | Colored output |
| `ui.emoji` | Emoji in output |
| `ui.verbose` | Verbose mode |
| `ui.date_format` | Date format string |

---

## `torii remote`

Create and manage remote repositories via platform APIs (requires auth token configured).

```bash
torii remote create github myrepo --public           # Create public repo
torii remote create gitlab myrepo --private          # Create private repo
torii remote create github myrepo --private --push   # Create + push current branch
torii remote delete github owner myrepo --yes        # Delete repo
torii remote visibility github owner myrepo --public # Make public
torii remote visibility github owner myrepo --private # Make private
torii remote configure github owner myrepo --default-branch main
torii remote info github owner myrepo                # Show repo details
torii remote list github                             # List your repos
```

---

## `torii repo`

Batch create/delete repos across multiple platforms at once.

```bash
torii repo myrepo --platforms github,gitlab,codeberg --create --private
torii repo myrepo --platforms github,gitlab --delete --yes
torii repo myrepo --platforms github,gitlab --create --public --push
```

---

## `.toriignore`

Works like `.gitignore` but syncs to `.git/info/exclude` automatically on every `torii open`. Patterns are respected by all git operations without committing ignore rules to the repo.

---

## System dependencies

> **Important:** Gitorii requires these system libraries to build from source:

| Platform | Command |
|----------|---------|
| Ubuntu/Debian | `apt install perl libssl-dev pkg-config` |
| Fedora/RHEL | `dnf install perl openssl-devel pkgconfig` |
| macOS | `brew install openssl pkg-config` |
| Arch | `pacman -S perl openssl pkgconf` |

---

## License

TSAL-1.0 — Free for personal and non-production use. Commercial use requires a license.  
See [LICENSE](LICENSE) for details. Converts to Apache 2.0 after 10 years.
