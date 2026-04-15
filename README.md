# Torii ⛩️

A human-first Git client. Simpler commands, built-in safety nets, and multi-platform support — designed for developers who want to focus on code, not version control syntax.

> Git was designed for Linus, by Linus. Torii is designed for everyone.

## Install

```bash
cargo install torii
```

Or build from source:

```bash
git clone https://gitlab.com/paskidev/torii.git
cd torii
cargo install --path .
```

## Quick start

```bash
# Save your work (replaces git add + git commit)
torii save -am "feat: add user auth"

# Stage specific files only
torii save src/auth.rs tests/auth.rs -m "feat: add user auth"

# Sync with remote (pull + push)
torii sync

# Push only
torii sync --push
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
| `torii save --reset <hash> --reset-mode soft` | Reset to commit |
| `torii sync` | Pull and push |
| `torii sync --push` | Push only |
| `torii sync --pull` | Pull only |
| `torii sync --force` | Force push |
| `torii sync --fetch` | Fetch without merging |
| `torii sync <branch>` | Integrate branch |
| `torii status` | Repository status |
| `torii diff` | Show unstaged changes |
| `torii diff --staged` | Show staged changes |
| `torii diff --last` | Show last commit diff |

### Branches

```bash
torii branch                  # list local branches
torii branch --all            # list local and remote branches
torii branch feature -c       # create and switch
torii branch main             # switch to branch
torii branch -d old-branch    # delete branch
torii branch --rename new-name
```

### History

```bash
torii log                          # last 10 commits
torii log -n 50                    # last 50 commits
torii log --oneline                # compact view
torii log --author "Pasqual"       # filter by author
torii log --since 2026-01-01       # filter by date
torii log --grep "feat"            # filter by message
torii log --stat                   # show file change stats
torii history reflog               # HEAD movement history
torii history rewrite "start" "end"  # rewrite commit dates
torii history clean                # gc + reflog expire
```

### Rebase

```bash
torii rebase main                  # rebase onto branch
torii rebase -i HEAD~5             # interactive rebase
torii rebase HEAD~5 --todo-file plan.txt  # rebase with pre-written todo
torii rebase --continue
torii rebase --abort
torii rebase --skip
```

### Snapshots

Snapshots are local saves — not commits. Use them before risky operations.

```bash
torii snapshot create -n "before-refactor"
torii snapshot list
torii snapshot restore <id>
torii snapshot delete <id>
torii snapshot stash              # stash current work
torii snapshot stash -u           # include untracked files
torii snapshot unstash
torii snapshot undo               # undo last operation
torii undo                        # shortcut
```

### Tags

```bash
torii tag create v1.0.0 -m "Release"
torii tag list
torii tag delete v1.0.0
torii tag push v1.0.0
torii tag show v1.0.0
torii tag release                 # auto-bump from commits since last tag
torii tag release --bump minor    # force bump type
torii tag release --dry-run       # preview without creating
```

`torii tag release` reads your commits since the last tag and bumps the version following [Conventional Commits](https://www.conventionalcommits.org/):
- `feat:` → minor bump
- `fix:` / `perf:` → patch bump
- `feat!:` / breaking → major bump

### Security scanner

```bash
torii scan                  # scan staged files before committing
torii scan --history        # scan entire git history
```

Runs automatically before every `torii save`. Detects JWT tokens, AWS keys, GitHub/GitLab tokens, Stripe keys, PEM private keys, database connection strings with credentials, and generic API keys. Files named `*.example`, `*.sample`, or `*.template` are always allowed.

### Mirrors

Mirror your repository across multiple platforms simultaneously.

```bash
torii mirror add-master github user myrepo user
torii mirror add-slave gitlab user myrepo user
torii mirror add-slave codeberg user myrepo user
torii mirror sync
torii mirror list
torii mirror remove <name>
torii mirror autofetch --enable --interval 30m
```

Supported platforms: GitHub, GitLab, Codeberg, Bitbucket, Gitea, Forgejo, SourceHut, SourceForge, and any custom Git server.

### Remote repository management

Create and manage repositories directly from the CLI:

```bash
torii remote create github myrepo --public
torii remote delete github owner myrepo --yes
torii remote visibility github owner myrepo --public
torii remote info github owner myrepo
torii remote list github

# Batch operations across platforms
torii repo myrepo --platforms github,gitlab --create --public
torii repo myrepo --platforms github,gitlab --delete --yes
```

### Other commands

```bash
torii clone github user/repo      # clone with platform shorthand
torii clone https://...           # clone with full URL
torii cherry-pick <hash>          # apply commit to current branch
torii blame src/main.rs           # line-by-line change history
torii blame src/main.rs -L 10,20  # specific line range
torii tag release                 # cut a release
torii ssh-check                   # verify SSH setup
torii custom add <name> <cmd>     # save custom workflow
torii custom run <name>           # run custom workflow
torii config set user.name "..."  # global or local config
```

## Why Torii?

| Git | Torii |
|-----|-------|
| `git add . && git commit -m "msg"` | `torii save -am "msg"` |
| `git pull && git push` | `torii sync` |
| `git switch -c branch` | `torii branch branch -c` |
| `git fetch` | `torii sync --fetch` |
| `git reset --soft HEAD~1 -m "msg"` | `torii save --reset HEAD~1 --reset-mode soft -m "msg"` |
| `git rebase -i HEAD~3` | `torii rebase -i HEAD~3` |
| `git stash push -u` | `torii snapshot stash -u` |
| Push to 3 platforms | `torii mirror sync` |
| Hunt for exposed secrets | `torii scan --history` |

## Links

- [Website](https://gitorii.com)
- [Issues](https://gitlab.com/paskidev/torii/-/issues)
- [License](LICENSE)
