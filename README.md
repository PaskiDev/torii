# Torii 🎌

A modern Git client with simplified commands and advanced features for developers who want power without complexity.

## ✨ Features

### 🎯 Simplified Commands
- **`torii save`** - Simplified commit (replaces `git add` + `git commit`)
- **`torii sync`** - Smart push/pull in one command
- **`torii switch`** - Easy branch switching and creation
- **`torii clone`** - Clone with platform shortcuts (e.g., `torii clone github user/repo`)

### 📸 Snapshot Management
- Create snapshots of your work at any time
- Restore to previous states easily
- Auto-snapshot configuration
- Stash/unstash functionality

### 🔄 Multi-Platform Mirrors
- Sync your repository across GitHub, GitLab, Bitbucket, and Codeberg
- Configure master/slave mirror relationships
- Automatic synchronization
- Protocol auto-detection (SSH/HTTPS)

### 🚀 Portable CI/CD
- Generate CI/CD configurations for multiple platforms
- Import existing configurations
- Validate and sync across platforms
- Support for GitHub Actions, GitLab CI, and more

### 🔧 Advanced Git Operations
- **Smart merge/rebase** - `torii integrate` analyzes your branch and recommends merge or rebase
- **Tag management** - Create, list, delete, push, and show tags
- **Cherry-pick** - Apply specific commits to current branch with conflict resolution
- **Blame** - See who changed each line of a file with line range support
- **History rewriting** - Rewrite commit dates and clean repository history

## 📦 Installation

### From Source
```bash
git clone https://github.com/yourusername/torii.git
cd torii
cargo build --release
```

### Using Cargo
```bash
cargo install torii
```

## 🚀 Quick Start

### Initialize a Repository
```bash
torii init
```

### Save Your Work
```bash
# Add all changes and commit
torii save -am "Initial commit"

# Amend previous commit
torii save --amend -m "Updated commit"

# Revert a specific commit
torii save --revert abc123 -m "Revert changes"
```

### Sync with Remote
```bash
# Pull and push in one command
torii sync

# Pull only
torii sync --pull

# Push only
torii sync --push

# Force push
torii sync --force

# Fetch only (update refs without merging)
torii sync --fetch
```

### Branch Management
```bash
# List branches
torii branch

# Create and switch to new branch
torii switch -c feature-x

# Switch to existing branch
torii switch main

# Delete branch
torii branch -d old-feature
```

### Clone Repositories
```bash
# Clone with platform shorthand
torii clone github facebook/react
torii clone gitlab user/project
torii clone codeberg user/repo

# Clone with full URL
torii clone https://github.com/user/repo.git
```

## 📚 Command Reference

### Basic Commands
- `torii init` - Initialize a new repository
- `torii save` - Save changes (commit, amend, revert)
- `torii sync` - Synchronize with remote (push, pull, fetch, force)
- `torii status` - Show repository status
- `torii log` - View commit history
- `torii diff` - Show changes
- `torii branch` - Manage branches
- `torii switch` - Switch branches
- `torii clone` - Clone repository
- `torii undo` - Undo last operation (quick access)

### Advanced Git Commands
- `torii integrate` - Smart merge/rebase integration
- `torii cherry-pick` - Apply a commit to current branch
- `torii blame` - Show who changed each line of a file

### Tag Commands
- `torii tag create` - Create a new tag
- `torii tag list` - List all tags
- `torii tag delete` - Delete a tag
- `torii tag push` - Push tags to remote
- `torii tag show` - Show tag details

### Snapshot Commands
- `torii snapshot create` - Create a snapshot
- `torii snapshot list` - List all snapshots
- `torii snapshot restore` - Restore from snapshot
- `torii snapshot delete` - Delete a snapshot
- `torii snapshot stash` - Stash current work
- `torii snapshot unstash` - Restore stashed work
- `torii snapshot undo` - Undo last operation

### Mirror Commands
- `torii mirror add-master` - Add master mirror
- `torii mirror add-slave` - Add slave mirror
- `torii mirror list` - List all mirrors
- `torii mirror sync` - Synchronize mirrors
- `torii mirror set-master` - Change master mirror
- `torii mirror remove` - Remove a mirror

### History Commands
- `torii history rewrite` - Rewrite commit dates
- `torii history clean` - Clean repository (gc, reflog)
- `torii history verify-remote` - Verify remote status

### CI/CD Commands
- `torii ci validate` - Validate CI/CD configuration
- `torii ci generate` - Generate CI/CD configs
- `torii ci import` - Import existing configs
- `torii ci sync` - Sync configs across platforms
- `torii ci diff` - Show config differences

### Utility Commands
- `torii ssh-check` - Check SSH configuration
- `torii help` - Show help information

## 🔐 SSH Configuration

Torii automatically detects SSH keys and uses the appropriate protocol:

```bash
# Check your SSH setup
torii ssh-check
```

If you don't have SSH keys, Torii will guide you through the setup process.

## 🌟 Examples

### Complete Workflow
```bash
# Clone a project
torii clone github facebook/react

# Create a feature branch
cd react
torii switch -c fix-bug-123

# Make changes and save
echo "fix" > bugfix.js
torii save -am "Fix bug #123"

# View your changes
torii diff --last

# Sync with remote
torii sync
```

### Multi-Platform Mirroring
```bash
# Set up GitHub as master
torii mirror add-master github user myproject myrepo

# Add GitLab as slave
torii mirror add-slave gitlab user myproject myrepo

# Sync all mirrors
torii mirror sync
```

### Snapshot Management
```bash
# Create a snapshot before risky changes
torii snapshot create -n "before-refactor"

# Make changes...

# If something goes wrong, restore
torii snapshot restore <snapshot-id>
```

### Smart Integration
```bash
# Preview merge/rebase recommendation
torii integrate feature-branch --preview

# Integrate with recommended strategy
torii integrate feature-branch

# Force merge even if rebase is recommended
torii integrate feature-branch --merge
```

### Tag Management
```bash
# Create an annotated tag
torii tag create v1.0.0 -m "Release version 1.0.0"

# List all tags
torii tag list

# Push tags to remote
torii tag push v1.0.0

# Show tag details
torii tag show v1.0.0
```

### Cherry-Pick and Blame
```bash
# Apply a specific commit to current branch
torii cherry-pick abc123

# Show who changed each line
torii blame src/main.rs

# Show blame for specific line range
torii blame src/main.rs -L 10,20
```

## 🎯 Why Torii?

### Simplified Workflow
Git commands can be complex and verbose. Torii simplifies common operations:

| Git | Torii |
|-----|-------|
| `git add . && git commit -m "msg"` | `torii save -am "msg"` |
| `git pull && git push` | `torii sync` |
| `git switch -c branch` | `torii switch -c branch` |
| `git clone https://...` | `torii clone github user/repo` |

### Advanced Features
- **Snapshots**: Time-travel through your work
- **Multi-Mirror**: Keep your code synced across platforms
- **Portable CI/CD**: One configuration, multiple platforms
- **Smart Operations**: Auto-detect best practices

### Developer-Friendly
- Clear, concise commands
- Helpful error messages
- Auto-detection of SSH/HTTPS
- Built-in best practices

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📄 License

MIT License - see LICENSE file for details

## 🔗 Links

- [Documentation](docs/)
- [Issue Tracker](https://github.com/yourusername/torii/issues)
- [Changelog](CHANGELOG.md)

---

**Torii** - Simplifying Git, one command at a time 🎌
