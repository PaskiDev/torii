# Gitorii ⛩️

A human-first Git client. Simpler commands, built-in safety nets, and multi-platform support — designed for developers who want to focus on code, not version control syntax.

> Git was designed for Linus, by Linus. Gitorii is designed for everyone — including AI.

## Install

**Linux / macOS — one line:**

```bash
curl -fsSL https://gitorii.com/install.sh | sh
```

**Windows — download from [gitorii.com/releases](https://gitorii.com/releases)**

**Via cargo (builds from source):**

```bash
cargo install gitorii
```

> **⚠️ Building from source requires system dependencies:**
>
> | Platform | Command |
> |----------|---------|
> | Ubuntu/Debian | `sudo apt install perl libssl-dev pkg-config` |
> | Fedora/RHEL | `sudo dnf install perl openssl-devel pkg-config` |
> | macOS | `brew install openssl` |
> | Arch | `sudo pacman -S perl openssl pkgconf` |

## Quick start

```bash
torii init                            # initialize repo
torii status                          # see what changed
torii save -am "feat: add user auth"  # stage all + commit
torii sync                            # pull + push
```

## Command reference

### Core

| Command | Description |
|---------|-------------|
| `torii init` | Initialize a repository |
| `torii save -m "msg"` | Commit staged changes |
| `torii save -am "msg"` | Stage all and commit |
| `torii save <files> -m "msg"` | Stage specific files and commit |
| `torii save --amend -m "msg"` | Amend last commit |
| `torii save --revert <hash> -m "msg"` | Revert a commit |
| `torii save --reset HEAD~1 --reset-mode soft` | Undo last commit, keep changes |
| `torii save --reset HEAD~1 --reset-mode hard` | Undo last commit, discard changes |
| `torii sync` | Pull and push |
| `torii sync --push` | Push only |
| `torii sync --pull` | Pull only |
| `torii sync --force` | Force push |
| `torii sync --fetch` | Fetch without merging |
| `torii sync <branch>` | Integrate branch (smart merge/rebase) |
| `torii sync <branch> --merge` | Force merge strategy |
| `torii sync <branch> --rebase` | Force rebase strategy |
| `torii sync <branch> --preview` | Preview without executing |
| `torii status` | Repository status |
| `torii diff` | Show unstaged changes |
| `torii diff --staged` | Show staged changes |
| `torii diff --last` | Show last commit diff |

### Workspaces

Run commands across multiple repos at once.

```bash
torii workspace add <name> ~/repos/api      # add repo to workspace
torii workspace add <name> ~/repos/frontend
torii workspace list                        # list all workspaces
torii workspace status <name>              # git status across all repos
torii workspace save <name> -m "wip" --all # commit all repos with changes
torii workspace sync <name>                # pull + push all repos
torii workspace remove <name> ~/repos/api  # remove a repo
torii workspace delete <name>              # delete workspace
```

### Branches

```bash
torii branch                  # list local branches
torii branch --all            # list local and remote branches
torii branch <name> -c        # create and switch
torii branch <name>           # switch to branch
torii branch -d <name>        # delete branch
torii branch --rename <name>  # rename current branch
```

### Tracked files

```bash
torii ls              # list all tracked files
torii ls src/         # filter by path prefix
```

### Inspect

```bash
torii show                         # show HEAD commit with diff
torii show <hash>                  # show specific commit
torii show <tag>                   # show tag details
torii show <file> --blame          # line-by-line change history
torii show <file> --blame -L 10,20 # specific line range
```

### History

```bash
torii log                           # last 10 commits
torii log -n 50                     # last 50 commits
torii log --oneline                 # compact view
torii log --graph                   # branch graph
torii log --author "Alice"          # filter by author
torii log --since 2026-01-01        # filter by date
torii log --grep "feat"             # filter by message
torii log --stat                    # show file change stats

torii history reflog                # HEAD movement history
torii history rewrite "2026-01-01" "2026-03-01"  # rewrite commit dates
torii history clean                 # expire reflogs + remove backup refs
torii history verify-remote         # verify remote state
torii history remove-file <path>    # purge file from entire history

torii history rebase main           # rebase onto branch
torii history rebase HEAD~5 -i      # interactive rebase (opens editor)
torii history rebase HEAD~5 --todo-file plan.txt
torii history rebase --continue
torii history rebase --abort
torii history rebase --skip

torii history cherry-pick <hash>    # apply commit to current branch
torii history cherry-pick --continue
torii history cherry-pick --abort

torii history blame <file>          # line-by-line change history
torii history blame <file> -L 10,20
```

### Security scanner

```bash
torii history scan            # scan staged files for secrets
torii history scan --history  # scan entire git history
```

Runs automatically before every `torii save`. Detects:
- JWT tokens, AWS keys (AKIA/ASIA), GitHub/GitLab tokens
- Stripe live keys, Twilio/SendGrid/Brevo keys
- PEM private keys, database connection strings with credentials
- Generic API keys and passwords

Files named `*.example`, `*.sample`, or `*.template` are always skipped.

### Snapshots

Snapshots are local saves — not commits. Use them before risky operations.

```bash
torii snapshot create -n "before-refactor"
torii snapshot list
torii snapshot restore <id>
torii snapshot delete <id>
torii snapshot stash              # quick stash
torii snapshot stash -u           # include untracked files
torii snapshot unstash
torii snapshot unstash <id> --keep
torii snapshot undo               # undo last operation
```

### Tags

```bash
torii tag create v1.0.0 -m "Release"
torii tag list
torii tag delete v1.0.0
torii tag push v1.0.0
torii tag push                    # push all tags
torii tag show v1.0.0
torii tag release                 # auto-bump from conventional commits
torii tag release --bump minor    # force bump type
torii tag release --dry-run       # preview without creating
```

`torii tag release` reads commits since the last tag and bumps following [Conventional Commits](https://www.conventionalcommits.org/):
- `feat:` → minor bump
- `fix:` / `perf:` → patch bump
- `feat!:` / breaking → major bump

### Mirrors

Mirror your repository across multiple platforms simultaneously.

```bash
torii mirror add-primary gitlab user <username> <repo>
torii mirror add-replica github user <username> <repo>
torii mirror add-replica codeberg user <username> <repo>
torii mirror sync
torii mirror sync --force
torii mirror list
torii mirror set-primary gitlab user
torii mirror remove github user
torii mirror autofetch --enable --interval 30m
torii mirror autofetch --disable
torii mirror autofetch --status
```

Supported platforms: GitHub, GitLab, Codeberg, Bitbucket, Gitea, Forgejo.

### Remote repository management

Create and manage repositories directly from the CLI (requires auth token in config):

```bash
torii remote create github <repo> --public
torii remote create github <repo> --private --description "My repo"
torii remote delete github <owner> <repo> --yes
torii remote visibility github <owner> <repo> --public
torii remote configure github <owner> <repo> --default-branch main
torii remote info github <owner> <repo>
torii remote list github

# Batch across platforms
torii repo <name> --platforms github,gitlab --create --public
torii repo <name> --platforms github,gitlab --delete --yes
```

### Config

```bash
torii config set user.name "Alice"
torii config set user.name "Alice" --local
torii config get user.name
torii config list
torii config list --local
torii config edit
torii config reset
```

Available keys: `user.name`, `user.email`, `user.editor`, `auth.github_token`, `auth.gitlab_token`, `git.default_branch`, `git.sign_commits`, `git.pull_rebase`, `mirror.default_protocol`, `snapshot.auto_enabled`, `ui.colors`, `ui.emoji`, `ui.verbose`.

### Other

```bash
torii clone github <user>/<repo>        # clone with platform shorthand
torii clone https://...                 # clone with full URL
torii clone github <user>/<repo> -d dir # clone into specific directory
torii ssh-check                         # verify SSH key setup
```

## Gitorii vs other Git clients

| Feature | Gitorii | Lazygit | GitUI | Tig | Magit | gh CLI |
|---------|:-------:|:-------:|:-----:|:---:|:-----:|:------:|
| Pure CLI (no TUI required) | ✓ | ✗ | ✗ | ✗ | ✗ | ✓ |
| Secret scanner (pre-commit) | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ |
| Scan full git history | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ |
| Snapshots (pre-op safety saves) | ✓ | ✗ | ✗ | ✗ | ~ | ✗ |
| Multi-remote mirrors | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ |
| Workspace (multi-repo commands) | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ |
| PR / MR creation from CLI | ✓ | ~ | ✗ | ✗ | ~ | ✓ |
| GitHub + GitLab native support | ✓ | ✗ | ✗ | ✗ | ~ | ✗ |
| Conventional commits auto-tag | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ |
| Remove file from entire history | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ |
| Interactive rebase | ✓ | ✓ | ~ | ✗ | ✓ | ✗ |
| No runtime dependencies | ✓ | ✗ | ✓ | ✓ | ✗ | ✗ |

> ✓ supported · ~ partial · ✗ not supported  
> Full comparison at [gitorii.com/vs](https://gitorii.com/vs)

## Why Gitorii?

| Git | Gitorii |
|-----|---------|
| `git add . && git commit -m "msg"` | `torii save -am "msg"` |
| `git pull && git push` | `torii sync` |
| `git switch -c branch` | `torii branch <name> -c` |
| `git fetch` | `torii sync --fetch` |
| `git reset --soft HEAD~1` | `torii save --reset HEAD~1 --reset-mode soft` |
| `git rebase -i HEAD~3` | `torii history rebase HEAD~3 -i` |
| `git stash push -u` | `torii snapshot stash -u` |
| `git log --oneline --author X` | `torii log --oneline --author X` |
| `git ls-files` | `torii ls` |
| `git show HEAD` | `torii show` |
| `git blame src/main.rs` | `torii show src/main.rs --blame` |
| Push to 3 platforms | `torii mirror sync` |
| Hunt for exposed secrets | `torii history scan --history` |
| Run status across 5 repos | `torii workspace status <name>` |
| Commit all dirty repos at once | `torii workspace save <name> -am "wip"` |

## System dependencies

Required to build from source. Pre-built binaries have no dependencies.

| Platform | Command |
|----------|---------|
| Ubuntu/Debian | `sudo apt install perl libssl-dev pkg-config` |
| Fedora/RHEL | `sudo dnf install perl openssl-devel pkg-config` |
| macOS | `brew install openssl` |
| Arch | `sudo pacman -S perl openssl pkgconf` |

## Links

- [Website](https://gitorii.com)
- [Releases](https://gitorii.com/releases)
- [Docs](https://gitorii.com/docs)
- [Issues](https://gitlab.com/paskidev/torii/-/issues)
- [crates.io](https://crates.io/crates/gitorii)

## License

TSAL-1.0 — Free for personal and non-production use. Commercial use requires a license. Converts to Apache 2.0 after 10 years. See [LICENSE](LICENSE) for details.
