use clap::{Parser, Subcommand};
use anyhow::Result;
use std::path::PathBuf;
use dirs;
use crate::config::ToriiConfig;
use crate::core::GitRepo;
use crate::remote::{get_platform_client, Visibility, RepoSettings, RepoFeatures};
use crate::snapshot::SnapshotManager;
use crate::mirror::{MirrorManager, AccountType, Protocol};
use crate::ssh::SshHelper;
use crate::duration::parse_duration;
use crate::versioning::AutoTagger;
use crate::scanner;

fn parse_account_type(s: &str) -> Result<AccountType> {
    match s.to_lowercase().as_str() {
        "user" | "u" => Ok(AccountType::User),
        "org" | "organization" | "o" => Ok(AccountType::Organization),
        _ => Err(anyhow::anyhow!("Invalid account type. Use 'user' or 'org'")),
    }
}

fn parse_protocol(s: Option<&String>) -> Protocol {
    match s.map(|s| s.to_lowercase()) {
        Some(p) if p == "https" || p == "http" => Protocol::HTTPS,
        Some(p) if p == "ssh" => Protocol::SSH,
        None => {
            // Auto-detect: use SSH if keys available, otherwise HTTPS
            if SshHelper::has_ssh_keys() {
                Protocol::SSH
            } else {
                println!("⚠️  No SSH keys detected. Using HTTPS protocol.");
                println!("   Run 'torii ssh-check' for SSH setup instructions.\n");
                Protocol::HTTPS
            }
        }
        _ => Protocol::SSH,
    }
}

#[derive(Parser)]
#[command(name = "torii")]
#[command(version, about = "A modern git client with simplified commands")]
#[command(after_help = "Examples:
  torii init                          Initialize a new repo
  torii save -am \"feat: add login\"    Stage all and commit
  torii sync                          Pull and push
  torii sync main                     Integrate main into current branch
  torii branch feature/auth -c        Create and switch to branch
  torii clone github user/repo        Clone from GitHub
  torii log --oneline --graph         Show compact history graph
  torii snapshot stash                Stash work in progress
  torii mirror sync                   Push to all configured mirrors

Run 'torii <command> --help' for detailed usage of any command.")]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new repository
    #[command(after_help = "Examples:
  torii init               Initialize in current directory
  torii init --path ~/projects/myrepo   Initialize in specific path")]
    Init {
        /// Path to initialize (defaults to current directory)
        #[arg(short, long)]
        path: Option<String>,
    },

    /// Save current work (simplified commit)
    #[command(after_help = "Examples:
  torii save -m \"fix: null check\"              Commit staged changes
  torii save -am \"feat: add login\"             Stage all and commit
  torii save src/auth.rs -m \"fix: token\"       Stage specific file and commit
  torii save --amend -m \"fix: typo\"            Amend last commit message
  torii save --revert abc1234 -m \"revert\"      Revert a specific commit
  torii save --reset HEAD~1 --reset-mode soft  Undo last commit, keep changes")]
    Save {
        /// Commit message
        #[arg(short, long)]
        message: String,

        /// Stage all changes before committing
        #[arg(short, long)]
        all: bool,

        /// Specific files to stage before committing
        #[arg(value_name = "FILES")]
        files: Vec<PathBuf>,

        /// Amend the previous commit
        #[arg(long)]
        amend: bool,

        /// Revert a specific commit by hash
        #[arg(long, value_name = "HASH")]
        revert: Option<String>,

        /// Reset to a specific commit (soft keeps changes staged, mixed unstages, hard discards)
        #[arg(long, value_name = "HASH")]
        reset: Option<String>,

        /// Reset mode: soft, mixed, or hard (default: mixed)
        #[arg(long, default_value = "mixed")]
        reset_mode: String,
    },

    /// Sync with remote (pull+push) or integrate a branch
    #[command(after_help = "Examples:
  torii sync                    Pull from remote then push
  torii sync --pull             Pull only
  torii sync --push             Push only
  torii sync --force            Force push (rewrites remote history)
  torii sync --fetch            Fetch remote refs without merging
  torii sync main               Integrate main into current branch (smart merge/rebase)
  torii sync main --merge       Force merge strategy
  torii sync main --rebase      Force rebase strategy
  torii sync main --preview     Preview what would happen without executing")]
    Sync {
        /// Branch to integrate (smart merge/rebase). If omitted, syncs with remote
        branch: Option<String>,

        /// Pull only
        #[arg(short, long)]
        pull: bool,

        /// Push only
        #[arg(short = 'P', long)]
        push: bool,

        /// Force push (rewrites remote history — use with caution)
        #[arg(short, long)]
        force: bool,

        /// Fetch remote refs without merging
        #[arg(long)]
        fetch: bool,

        /// Force merge strategy when integrating a branch
        #[arg(long)]
        merge: bool,

        /// Force rebase strategy when integrating a branch
        #[arg(long)]
        rebase: bool,

        /// Preview integration without executing
        #[arg(long)]
        preview: bool,
    },

    /// Show repository status
    #[command(after_help = "Examples:
  torii status    Show staged, unstaged, and untracked files")]
    Status,

    /// Show commit history
    #[command(after_help = "Examples:
  torii log                          Last 10 commits
  torii log -n 50                    Last 50 commits
  torii log --oneline                One line per commit
  torii log --graph                  Branch graph
  torii log --oneline --graph        Compact graph view
  torii log --author \"Alice\"         Filter by author
  torii log --since 2024-01-01       Commits after date
  torii log --until 2024-12-31       Commits before date
  torii log --grep \"feat\"            Filter by message pattern
  torii log --stat                   Show file change stats per commit")]
    Log {
        /// Number of commits to show (default: 10)
        #[arg(short = 'n', long)]
        count: Option<usize>,

        /// Show one line per commit
        #[arg(long)]
        oneline: bool,

        /// Show branch graph
        #[arg(long)]
        graph: bool,

        /// Filter by author name or email
        #[arg(long)]
        author: Option<String>,

        /// Show commits after this date (YYYY-MM-DD)
        #[arg(long)]
        since: Option<String>,

        /// Show commits before this date (YYYY-MM-DD)
        #[arg(long)]
        until: Option<String>,

        /// Filter commits whose message matches this pattern
        #[arg(long)]
        grep: Option<String>,

        /// Show file change statistics per commit
        #[arg(long)]
        stat: bool,
    },

    /// Show unstaged or staged changes
    #[command(after_help = "Examples:
  torii diff            Show unstaged changes
  torii diff --staged   Show staged changes (ready to commit)
  torii diff --last     Show changes in last commit")]
    Diff {
        /// Show staged changes
        #[arg(long)]
        staged: bool,

        /// Show last commit diff
        #[arg(long)]
        last: bool,
    },

    /// Manage branches
    #[command(after_help = "Examples:
  torii branch                      List local branches
  torii branch --all                List local and remote branches
  torii branch feature/auth -c      Create and switch to branch
  torii branch main                 Switch to existing branch
  torii branch -d feature/auth      Delete branch
  torii branch --rename new-name    Rename current branch")]
    Branch {
        /// Branch name to switch to or create with -c
        name: Option<String>,

        /// Create new branch and switch to it
        #[arg(short, long)]
        create: bool,

        /// Delete branch by name
        #[arg(short, long)]
        delete: Option<String>,

        /// List local branches
        #[arg(short, long)]
        list: bool,

        /// Rename current branch to this name
        #[arg(short, long)]
        rename: Option<String>,

        /// Show all branches including remote
        #[arg(short, long)]
        all: bool,
    },

    /// Clone a repository
    #[command(after_help = "Examples:
  torii clone github user/repo              Clone from GitHub (auto SSH/HTTPS)
  torii clone gitlab user/repo              Clone from GitLab
  torii clone github user/repo --protocol https   Force HTTPS
  torii clone github user/repo -d my-dir   Clone into specific directory
  torii clone https://github.com/user/repo.git    Clone from full URL
  torii clone git@github.com:user/repo.git        Clone via SSH URL

Supported platforms: github, gitlab, codeberg, bitbucket, gitea, forgejo

Protocol is auto-detected: SSH if keys are configured, HTTPS otherwise.
Override with --protocol or set default: torii config set mirror.default_protocol https")]
    Clone {
        /// Platform (github, gitlab, ...) or full URL (https://... / git@...)
        source: String,

        /// Repository as user/repo (when using platform shorthand)
        args: Vec<String>,

        /// Target directory name
        #[arg(short = 'd', long)]
        directory: Option<String>,

        /// Protocol to use: ssh or https (default: auto-detect)
        #[arg(long)]
        protocol: Option<String>,
    },

    /// Manage tags and releases
    #[command(after_help = "Examples:
  torii tag list                      List all tags
  torii tag create v1.2.0 -m \"Release\"   Create annotated tag
  torii tag delete v1.0.0             Delete a tag
  torii tag push v1.2.0               Push specific tag to remote
  torii tag push                      Push all tags to remote
  torii tag show v1.2.0               Show tag details
  torii tag release                   Auto-bump version from conventional commits
  torii tag release --bump minor      Force minor bump
  torii tag release --dry-run         Preview without creating tag

Auto-bump rules (Conventional Commits):
  feat:        → minor bump (0.1.0 → 0.2.0)
  fix: / perf: → patch bump (0.1.0 → 0.1.1)
  feat!:       → major bump (0.1.0 → 1.0.0)")]
    Tag {
        #[command(subcommand)]
        action: TagCommands,
    },

    /// Save and restore work-in-progress snapshots
    #[command(after_help = "Examples:
  torii snapshot create -n \"before-refactor\"   Create named snapshot
  torii snapshot list                           List all snapshots
  torii snapshot restore <id>                   Restore a snapshot
  torii snapshot delete <id>                    Delete a snapshot
  torii snapshot stash                          Stash current work
  torii snapshot stash -u                       Stash including untracked files
  torii snapshot unstash                        Restore latest stash
  torii snapshot unstash <id> --keep            Restore stash but keep it
  torii snapshot undo                           Undo last operation")]
    Snapshot {
        #[command(subcommand)]
        action: SnapshotCommands,
    },

    /// Mirror repository across multiple platforms
    #[command(after_help = "Examples:
  torii mirror add-master gitlab user paskidev myrepo    Set GitLab as primary
  torii mirror add-slave github user paskidev myrepo     Add GitHub as mirror
  torii mirror sync                                      Push to all mirrors
  torii mirror sync --force                              Force push to all mirrors
  torii mirror list                                      List configured mirrors
  torii mirror remove github user                        Remove a mirror
  torii mirror autofetch --enable --interval 30m         Auto-fetch every 30 min
  torii mirror autofetch --disable                       Disable auto-fetch
  torii mirror autofetch --status                        Show autofetch status

Supported platforms: github, gitlab, codeberg, bitbucket, gitea, forgejo")]
    Mirror {
        #[command(subcommand)]
        action: MirrorCommands,
    },

    /// List all tracked files
    #[command(after_help = "Examples:
  torii ls           List all tracked files
  torii ls src/      List tracked files under src/")]
    Ls {
        /// Filter by path prefix (e.g. src/)
        path: Option<String>,
    },

    /// Show commit, tag, or file details
    #[command(after_help = "Examples:
  torii show                      Show HEAD commit with diff
  torii show abc1234              Show specific commit
  torii show v1.0.0               Show tag details
  torii show src/main.rs --blame  Show line-by-line change history
  torii show src/main.rs --blame -L 10,20   Blame specific line range")]
    Show {
        /// Commit hash, tag name, ref, or file path (defaults to HEAD)
        object: Option<String>,

        /// Show blame for a file (who changed each line)
        #[arg(long)]
        blame: bool,

        /// Line range for blame (e.g., 10,20)
        #[arg(short = 'L', long, requires = "blame")]
        lines: Option<String>,
    },

    /// Check and troubleshoot SSH configuration
    #[command(after_help = "Examples:
  torii ssh-check    Verify SSH keys and show setup instructions if needed")]
    SshCheck,

    /// Manage commit history (rebase, cherry-pick, blame, scan)
    #[command(after_help = "Examples:
  torii history reflog                        Show HEAD movement history
  torii history rebase main                   Rebase current branch onto main
  torii history rebase -i HEAD~5              Interactive rebase last 5 commits
  torii history rebase --continue             Continue after resolving conflicts
  torii history rebase --abort                Abort current rebase
  torii history cherry-pick abc1234           Apply a commit to current branch
  torii history blame src/main.rs             Line-by-line change history
  torii history blame src/main.rs -L 10,20    Specific line range
  torii history scan                          Scan staged files for secrets
  torii history scan --history                Scan entire git history for secrets
  torii history remove-file secrets.txt       Purge file from all commits
  torii history rewrite \"2024-01-01\" \"2024-12-31\"  Rewrite commit dates
  torii history clean                         GC and expire reflog")]
    History {
        #[command(subcommand)]
        action: HistoryCommands,
    },

    /// Manage Torii configuration
    #[command(after_help = "Examples:
  torii config list                              Show all config values
  torii config list --local                      Show local repo config
  torii config get user.name                     Get a value
  torii config set user.name \"Alice\"             Set a global value
  torii config set user.email \"a@b.com\" --local  Set a local value
  torii config set auth.github_token ghp_xxx     Set GitHub token
  torii config set auth.gitlab_token glpat-xxx   Set GitLab token
  torii config set mirror.default_protocol https Use HTTPS by default
  torii config edit                              Open config in editor
  torii config reset                             Reset to defaults

Available keys:
  user.name, user.email, user.editor
  auth.github_token, auth.gitlab_token, auth.gitea_token
  auth.forgejo_token, auth.codeberg_token
  git.default_branch, git.sign_commits, git.pull_rebase
  mirror.default_protocol, mirror.autofetch_enabled
  snapshot.auto_enabled, snapshot.auto_interval_minutes
  ui.colors, ui.emoji, ui.verbose, ui.date_format")]
    Config {
        #[command(subcommand)]
        action: ConfigCommands,
    },

    /// Manage remote repositories (create, delete, configure)
    #[command(after_help = "Examples:
  torii remote create github myrepo --public          Create public repo on GitHub
  torii remote create gitlab myrepo --private         Create private repo on GitLab
  torii remote create github myrepo --private --push  Create and push current branch
  torii remote delete github owner myrepo --yes        Delete repo (no confirmation)
  torii remote visibility github owner myrepo --public Make repo public
  torii remote visibility github owner myrepo --private Make repo private
  torii remote configure github owner myrepo --default-branch main
  torii remote info github owner myrepo               Show repo details
  torii remote list github                            List all your GitHub repos

Supported platforms: github, gitlab, codeberg, bitbucket, gitea, forgejo")]
    Remote {
        #[command(subcommand)]
        action: RemoteCommands,
    },

    /// Batch operations on multiple platforms
    Repo {
        /// Repository name
        name: String,
        
        /// Platforms (comma-separated: github,gitlab,codeberg)
        #[arg(long, value_delimiter = ',', num_args = 1..)]
        platforms: Vec<String>,
        
        /// Create repository
        #[arg(long)]
        create: bool,
        
        /// Delete repository
        #[arg(long)]
        delete: bool,
        
        /// Make public
        #[arg(long)]
        public: bool,
        
        /// Make private
        #[arg(long)]
        private: bool,
        
        /// Description
        #[arg(long)]
        description: Option<String>,
        
        /// Push after creation
        #[arg(long)]
        push: bool,
        
        /// Skip confirmation
        #[arg(short = 'y', long)]
        yes: bool,
        
        /// Owner/username
        #[arg(long)]
        owner: Option<String>,
    },

    /// Manage multi-repo workspaces
    #[command(after_help = "Examples:
  torii workspace add work ~/repos/api   Add repo to workspace
  torii workspace list                   List all workspaces
  torii workspace status work            Show status of all repos
  torii workspace save work -m \"wip\"    Commit across all repos
  torii workspace sync work              Pull+push all repos")]
    Workspace {
        #[command(subcommand)]
        action: WorkspaceCommands,
    },

    /// Open the interactive TUI dashboard
    #[command(after_help = "Examples:
  torii tui   Open dashboard (status, log, file navigation)")]
    Tui,
}

#[derive(Subcommand)]
enum WorkspaceCommands {
    /// Add a repository to a workspace
    Add {
        /// Workspace name
        workspace: String,
        /// Repository path
        path: String,
    },
    /// Remove a repository from a workspace
    Remove {
        /// Workspace name
        workspace: String,
        /// Repository path
        path: String,
    },
    /// Delete a workspace entirely
    Delete {
        /// Workspace name
        workspace: String,
    },
    /// List all workspaces and their repos
    List,
    /// Show git status across all repos in a workspace
    Status {
        /// Workspace name
        workspace: String,
    },
    /// Commit changes across all repos in a workspace
    Save {
        /// Workspace name
        workspace: String,
        /// Commit message
        #[arg(short, long)]
        message: String,
        /// Stage all changes before committing
        #[arg(short, long)]
        all: bool,
    },
    /// Pull and push all repos in a workspace
    Sync {
        /// Workspace name
        workspace: String,
        /// Force push
        #[arg(long)]
        force: bool,
    },
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Set a configuration value
    Set {
        /// Configuration key (e.g., user.name, snapshot.auto_enabled)
        key: String,
        
        /// Configuration value
        value: String,
        
        /// Set in local repository config instead of global
        #[arg(long)]
        local: bool,
    },
    
    /// Get a configuration value
    Get {
        /// Configuration key (e.g., user.name, snapshot.auto_enabled)
        key: String,
        
        /// Get from local repository config
        #[arg(long)]
        local: bool,
    },
    
    /// List all configuration values
    List {
        /// Show local repository config
        #[arg(long)]
        local: bool,
    },
    
    /// Edit configuration file in editor
    Edit {
        /// Edit local repository config instead of global
        #[arg(long)]
        local: bool,
    },
    
    /// Reset configuration to defaults
    Reset {
        /// Reset local repository config instead of global
        #[arg(long)]
        local: bool,
    },
}

#[derive(Subcommand)]
enum RemoteCommands {
    /// Create a new remote repository
    Create {
        platform: String,
        name: String,
        #[arg(short, long)]
        description: Option<String>,
        #[arg(long)]
        public: bool,
        #[arg(long)]
        private: bool,
        #[arg(long)]
        push: bool,
    },
    Delete {
        platform: String,
        owner: String,
        repo: String,
        #[arg(short = 'y', long)]
        yes: bool,
    },
    Visibility {
        platform: String,
        owner: String,
        repo: String,
        #[arg(long, conflicts_with = "private")]
        public: bool,
        #[arg(long, conflicts_with = "public")]
        private: bool,
    },
    Configure {
        platform: String,
        owner: String,
        repo: String,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        homepage: Option<String>,
        #[arg(long)]
        default_branch: Option<String>,
        #[arg(long)]
        enable_issues: bool,
        #[arg(long, conflicts_with = "enable_issues")]
        disable_issues: bool,
        #[arg(long)]
        enable_wiki: bool,
        #[arg(long, conflicts_with = "enable_wiki")]
        disable_wiki: bool,
        #[arg(long)]
        enable_projects: bool,
        #[arg(long, conflicts_with = "enable_projects")]
        disable_projects: bool,
    },
    Info {
        platform: String,
        owner: String,
        repo: String,
    },
    List {
        platform: String,
    },
}

#[derive(Subcommand)]
enum HistoryCommands {
    /// Rewrite commit history dates
    Rewrite {
        /// Start date (YYYY-MM-DD HH:MM)
        start: String,
        
        /// End date (YYYY-MM-DD HH:MM)
        end: String,
    },
    
    /// Clean repository history
    Clean,

    /// Verify remote repository
    VerifyRemote,

    /// Show reflog (reference log of HEAD movements)
    Reflog {
        /// Number of entries to show
        #[arg(short = 'n', long, default_value = "20")]
        count: usize,
    },

    /// Remove a file from the entire git history
    RemoveFile {
        /// File path to remove from all commits
        file: String,
    },

    /// Apply a commit from another branch to current branch
    CherryPick {
        /// Commit hash to cherry-pick
        commit: Option<String>,

        /// Continue after resolving conflicts
        #[arg(long)]
        r#continue: bool,

        /// Abort cherry-pick
        #[arg(long)]
        abort: bool,
    },

    /// Show who changed each line of a file
    Blame {
        /// File to blame
        file: String,

        /// Line range (e.g., 10,20)
        #[arg(short = 'L', long)]
        lines: Option<String>,
    },

    /// Scan staged files or full history for sensitive data
    Scan {
        /// Scan the entire git history instead of only staged files
        #[arg(long)]
        history: bool,
    },

    /// Rebase current branch onto a target
    Rebase {
        /// Target branch or commit to rebase onto
        target: Option<String>,

        /// Interactive rebase
        #[arg(short, long)]
        interactive: bool,

        /// Path to a pre-written rebase todo file (skips editor)
        #[arg(long, value_name = "FILE")]
        todo_file: Option<PathBuf>,

        /// Continue an in-progress rebase
        #[arg(long)]
        r#continue: bool,

        /// Abort the current rebase
        #[arg(long)]
        abort: bool,

        /// Skip the current patch
        #[arg(long)]
        skip: bool,
    },
}


#[derive(Subcommand)]
enum SnapshotCommands {
    /// Create a new snapshot
    Create {
        /// Optional snapshot name/description
        #[arg(short, long)]
        name: Option<String>,
    },

    /// List all snapshots
    List,

    /// Restore from a snapshot
    Restore {
        /// Snapshot ID to restore
        id: String,
    },

    /// Delete a snapshot
    Delete {
        /// Snapshot ID to delete
        id: String,
    },

    /// Auto-snapshot configuration
    Config {
        /// Enable auto-snapshots
        #[arg(long)]
        enable: bool,

        /// Snapshot interval (e.g., 1h, 30m)
        #[arg(long)]
        interval: Option<String>,
    },

    /// Save work temporarily (like git stash)
    Stash {
        /// Name for the stash
        #[arg(short, long)]
        name: Option<String>,

        /// Include untracked files
        #[arg(short = 'u', long)]
        include_untracked: bool,
    },

    /// Restore stashed work
    Unstash {
        /// Stash ID to restore (latest if not specified)
        id: Option<String>,

        /// Keep the stash after restoring
        #[arg(short, long)]
        keep: bool,
    },

    /// Undo last operation
    Undo,
}

#[derive(Debug, Subcommand)]
enum TagCommands {
    /// Create a new tag
    Create {
        /// Tag name
        name: String,

        /// Tag message (creates annotated tag)
        #[arg(short, long)]
        message: Option<String>,
    },

    /// List all tags
    List,

    /// Delete a tag
    Delete {
        /// Tag name to delete
        name: String,
    },

    /// Push tags to remote
    Push {
        /// Specific tag to push (all if not specified)
        name: Option<String>,
    },

    /// Show tag details
    Show {
        /// Tag name
        name: String,
    },

    /// Create the next release tag based on conventional commits since last tag
    Release {
        /// Force a specific bump: major, minor, patch
        #[arg(long)]
        bump: Option<String>,

        /// Preview the next version without creating the tag
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Subcommand)]
enum MirrorCommands {
    /// Add primary mirror (main repository)
    AddPrimary {
        /// Platform (github, gitlab, bitbucket, codeberg)
        platform: String,

        /// Account type (user or org)
        account_type: String,

        /// Account name (username or organization)
        account: String,

        /// Repository name
        repo: String,

        /// Protocol (ssh or https, defaults to ssh)
        #[arg(short, long)]
        protocol: Option<String>,
    },

    /// Add replica mirror (will sync from master)
    AddReplica {
        /// Platform (github, gitlab, bitbucket, codeberg)
        platform: String,

        /// Account type (user or org)
        account_type: String,

        /// Account name (username or organization)
        account: String,

        /// Repository name
        repo: String,

        /// Protocol (ssh or https, defaults to ssh)
        #[arg(short, long)]
        protocol: Option<String>,
    },

    /// List all mirrors
    List,

    /// Sync to all replica mirrors
    Sync {
        /// Force sync
        #[arg(short, long)]
        force: bool,
    },

    /// Set a mirror as primary
    SetPrimary {
        /// Platform
        platform: String,

        /// Account name
        account: String,
    },

    /// Remove a mirror
    Remove {
        /// Platform
        platform: String,

        /// Account name
        account: String,
    },

    /// Configure autofetch (automatic fetch from mirrors)
    Autofetch {
        /// Enable autofetch
        #[arg(long)]
        enable: bool,

        /// Disable autofetch
        #[arg(long, conflicts_with = "enable")]
        disable: bool,

        /// Fetch interval (e.g., 10m, 30s, 2h, 1d)
        #[arg(long)]
        interval: Option<String>,

        /// Show current autofetch status
        #[arg(long, conflicts_with_all = ["enable", "disable", "interval"])]
        status: bool,
    },
}

impl Cli {
    pub fn execute(&self) -> Result<()> {
        match &self.command {
            Commands::Init { path } => {
                let repo_path = path.as_deref().unwrap_or(".");
                GitRepo::init(repo_path)?;

                // Create .toriignore with sensible defaults
                let toriignore_path = std::path::Path::new(repo_path).join(".toriignore");
                if !toriignore_path.exists() {
                    std::fs::write(&toriignore_path, crate::toriignore::ToriIgnore::default_content())
                        .ok();
                }

                // Sync .toriignore → .git/info/exclude immediately
                let repo = GitRepo::open(repo_path)?;
                repo.sync_toriignore()?;

                println!("✅ Initialized repository at {}", repo_path);
                println!("   Created .toriignore with default patterns");
            }

            Commands::Save { message, all, files, amend, revert, reset, reset_mode } => {
                let repo = GitRepo::open(".")?;

                if let Some(commit_hash) = reset {
                    repo.reset_commit(commit_hash, reset_mode)?;
                    println!("✅ Reset to commit: {} (mode: {})", commit_hash, reset_mode);
                } else if let Some(commit_hash) = revert {
                    repo.revert_commit(commit_hash)?;
                    println!("✅ Reverted commit: {}", commit_hash);
                } else {
                    if *all && !files.is_empty() {
                        anyhow::bail!("Cannot use --all and specific files at the same time");
                    }
                    if *all {
                        repo.add_all()?;
                    } else if !files.is_empty() {
                        repo.add(files)?;
                    }
                    
                    // Scan staged files for sensitive data before committing
                    let repo_path = std::path::Path::new(".");
                    let findings = scanner::scan_staged(repo_path)?;
                    if !findings.is_empty() {
                        println!("⚠️  Sensitive data detected in staged files:\n");
                        for f in &findings {
                            if f.line == 0 {
                                println!("   {} — {}", f.file, f.pattern_name);
                            } else {
                                println!("   {}:{} — {}", f.file, f.line, f.pattern_name);
                            }
                            println!("   {}\n", f.preview);
                        }
                        println!("💡 Tip: use .env.example for placeholder values — those files are always safe to commit.");
                        print!("   Continue anyway? [y/N] ");
                        use std::io::Write;
                        std::io::stdout().flush()?;
                        let mut input = String::new();
                        std::io::stdin().read_line(&mut input)?;
                        if !input.trim().eq_ignore_ascii_case("y") {
                            println!("❌ Commit cancelled.");
                            return Ok(());
                        }
                    }

                    if *amend {
                        repo.commit_amend(message)?;
                        println!("✅ Commit amended: {}", message);
                    } else {
                        repo.commit(message)?;
                        println!("✅ Changes saved: {}", message);
                    }
                }
            }

            Commands::Sync { branch, pull, push, force, fetch, merge, rebase, preview } => {
                let repo = GitRepo::open(".")?;
                
                // If branch is specified, integrate it
                if let Some(branch_name) = branch {
                    if *preview {
                        println!("🔍 Preview: Would integrate branch '{}'", branch_name);
                        println!("💡 Recommendation: Use merge for feature branches, rebase for clean history");
                    } else if *merge {
                        println!("🔀 Merging branch '{}'...", branch_name);
                        repo.merge_branch(branch_name)?;
                        println!("✅ Merged branch: {}", branch_name);
                    } else if *rebase {
                        println!("🔄 Rebasing onto branch '{}'...", branch_name);
                        repo.rebase_branch(branch_name)?;
                        println!("✅ Rebased onto: {}", branch_name);
                    } else {
                        // Smart integration (default to merge for now)
                        println!("🔀 Integrating branch '{}'...", branch_name);
                        repo.merge_branch(branch_name)?;
                        println!("✅ Integrated branch: {}", branch_name);
                    }
                } else if *fetch {
                    repo.fetch()?;
                    println!("✅ Fetched from remote");
                } else if *force {
                    repo.push(true)?;
                    println!("✅ Force synced with remote");
                    let mirror_mgr = MirrorManager::new(".")?;
                    mirror_mgr.sync_replicas_if_any(true)?;
                } else if *pull {
                    repo.pull()?;
                    println!("✅ Pulled from remote");
                } else if *push {
                    repo.push(false)?;
                    println!("✅ Pushed to remote");
                    let mirror_mgr = MirrorManager::new(".")?;
                    mirror_mgr.sync_replicas_if_any(false)?;
                } else {
                    // Default: pull then push
                    repo.pull()?;
                    repo.push(false)?;
                    println!("✅ Synced with remote");
                    // Also sync replica mirrors if any are configured
                    let mirror_mgr = MirrorManager::new(".")?;
                    mirror_mgr.sync_replicas_if_any(false)?;
                }
            }

            Commands::Status => {
                let repo = GitRepo::open(".")?;
                repo.status()?;
            }

            Commands::Log { count, oneline, graph, author, since, until, grep, stat } => {
                let repo = GitRepo::open(".")?;
                repo.log(*count, *oneline, *graph, author.as_deref(), since.as_deref(), until.as_deref(), grep.as_deref(), *stat)?;
            }

            Commands::Diff { staged, last } => {
                let repo = GitRepo::open(".")?;
                repo.diff(*staged, *last)?;
            }

            Commands::Branch { name, create, delete, list, rename, all } => {
                let repo = GitRepo::open(".")?;
                
                if *list || *all {
                    let branches = repo.list_branches()?;
                    println!("📋 Branches:");
                    for branch in branches {
                        println!("  • {}", branch);
                    }
                    if *all {
                        let remote_branches = repo.list_remote_branches()?;
                        println!("\n📡 Remote branches:");
                        if remote_branches.is_empty() {
                            println!("  (none — run 'torii sync --fetch' to update remote refs)");
                        } else {
                            for branch in remote_branches {
                                println!("  • {}", branch);
                            }
                        }
                    }
                } else if let Some(branch_name) = delete {
                    repo.delete_branch(branch_name)?;
                    println!("✅ Deleted branch: {}", branch_name);
                } else if let Some(new_name) = rename {
                    let current = repo.get_current_branch()?;
                    repo.rename_branch(&current, new_name)?;
                    println!("✅ Renamed branch {} to {}", current, new_name);
                } else if let Some(branch_name) = name {
                    if *create {
                        // Create and switch to new branch
                        repo.create_branch(branch_name)?;
                        repo.switch_branch(branch_name)?;
                        println!("✅ Created and switched to branch: {}", branch_name);
                    } else {
                        // Just switch to existing branch
                        repo.switch_branch(branch_name)?;
                        println!("✅ Switched to branch: {}", branch_name);
                    }
                } else {
                    // Default: list branches
                    let branches = repo.list_branches()?;
                    println!("📋 Branches:");
                    for branch in branches {
                        println!("  • {}", branch);
                    }
                }
            }

            Commands::Clone { source, args, directory, protocol } => {
                let url = if !args.is_empty() {
                    // Shorthand: torii clone <platform> <user/repo>
                    let platform = source;
                    let user_repo = &args[0];

                    // Protocol priority: --protocol flag > config > auto-detect
                    let use_ssh = match protocol.as_deref() {
                        Some("https") | Some("http") => false,
                        Some("ssh") => true,
                        _ => {
                            let cfg = ToriiConfig::load_global().unwrap_or_default();
                            if cfg.mirror.default_protocol == "https" {
                                false
                            } else {
                                SshHelper::has_ssh_keys()
                            }
                        }
                    };

                    let (ssh_host, https_host) = match platform.as_str() {
                        "github"    => ("github.com", "github.com"),
                        "gitlab"    => ("gitlab.com", "gitlab.com"),
                        "codeberg"  => ("codeberg.org", "codeberg.org"),
                        "bitbucket" => ("bitbucket.org", "bitbucket.org"),
                        "gitea"     => ("gitea.com", "gitea.com"),
                        "forgejo"   => ("codeberg.org", "codeberg.org"),
                        _ => anyhow::bail!(
                            "Unknown platform '{}'. Supported: github, gitlab, codeberg, bitbucket, gitea, forgejo",
                            platform
                        ),
                    };

                    if use_ssh {
                        format!("git@{}:{}.git", ssh_host, user_repo)
                    } else {
                        format!("https://{}/{}.git", https_host, user_repo)
                    }
                } else if source.starts_with("http") || source.starts_with("git@") {
                    source.clone()
                } else {
                    anyhow::bail!(
                        "Usage:\n  torii clone <platform> <user/repo>        e.g. torii clone github user/repo\n  torii clone <platform> <user/repo> --protocol https\n  torii clone <url>                          e.g. torii clone https://github.com/user/repo.git"
                    )
                };

                let target_dir = directory.as_deref();
                GitRepo::clone_repo(&url, target_dir)?;

                let dir_name = target_dir.unwrap_or_else(|| {
                    url.split('/').last().unwrap_or("repo").trim_end_matches(".git")
                });
                println!("✅ Cloned repository to: {}", dir_name);
            }

            Commands::Tag { action } => {
                let repo = GitRepo::open(".")?;
                match action {
                    TagCommands::Create { name, message } => {
                        repo.create_tag(name, message.as_deref())?;
                        println!("✅ Tag created: {}", name);
                    }
                    TagCommands::List => {
                        repo.list_tags()?;
                    }
                    TagCommands::Delete { name } => {
                        repo.delete_tag(name)?;
                        println!("✅ Tag deleted: {}", name);
                    }
                    TagCommands::Push { name } => {
                        repo.push_tags(name.as_deref())?;
                        if let Some(tag) = name {
                            println!("✅ Pushed tag: {}", tag);
                        } else {
                            println!("✅ Pushed all tags");
                        }
                    }
                    TagCommands::Show { name } => {
                        repo.show_tag(name)?;
                    }
                    TagCommands::Release { bump, dry_run } => {
                        let tagger = AutoTagger::new(repo);
                        let current = tagger.get_latest_version()?;

                        let next = if let Some(bump_str) = bump {
                            use crate::versioning::semver::VersionBump;
                            let b = match bump_str.as_str() {
                                "major" => VersionBump::Major,
                                "minor" => VersionBump::Minor,
                                "patch" => VersionBump::Patch,
                                _ => anyhow::bail!("Invalid bump: use major, minor or patch"),
                            };
                            let base = current.unwrap_or_else(crate::versioning::semver::Version::initial);
                            base.bump(b)
                        } else {
                            // Infer bump from commits since last tag
                            tagger.calculate_next_version_from_log()?
                                .ok_or_else(|| anyhow::anyhow!("No releasable commits found since last tag (need feat: or fix:)"))?
                        };

                        println!("📦 Current version: {}", current.map(|v| v.to_string()).unwrap_or_else(|| "none".to_string()));
                        println!("🚀 Next version:    v{}", next);

                        if *dry_run {
                            println!("   (dry run — no tag created)");
                        } else {
                            tagger.create_tag(&next, &format!("Release v{}", next))?;
                            println!("💡 Push with: torii sync --push");
                        }
                    }
                }
            }

            Commands::Snapshot { action } => {
                let snapshot_mgr = SnapshotManager::new(".")?;
                match action {
                    SnapshotCommands::Create { name } => {
                        let snapshot_id = snapshot_mgr.create_snapshot(name.as_deref())?;
                        println!("✅ Snapshot created: {}", snapshot_id);
                    }
                    SnapshotCommands::List => {
                        snapshot_mgr.list_snapshots()?;
                    }
                    SnapshotCommands::Restore { id } => {
                        snapshot_mgr.restore_snapshot(id)?;
                        println!("✅ Restored snapshot: {}", id);
                    }
                    SnapshotCommands::Delete { id } => {
                        snapshot_mgr.delete_snapshot(id)?;
                        println!("✅ Deleted snapshot: {}", id);
                    }
                    SnapshotCommands::Config { enable, interval } => {
                        let interval_minutes = interval.as_ref().and_then(|s| s.parse::<u32>().ok());
                        snapshot_mgr.configure_auto_snapshot(*enable, interval_minutes)?;
                        println!("✅ Auto-snapshot configuration updated");
                    }
                    SnapshotCommands::Stash { name, include_untracked } => {
                        snapshot_mgr.stash(name.as_deref(), *include_untracked)?;
                    }
                    SnapshotCommands::Unstash { id, keep } => {
                        snapshot_mgr.unstash(id.as_deref(), *keep)?;
                    }
                    SnapshotCommands::Undo => {
                        snapshot_mgr.undo()?;
                    }
                }
            }

            Commands::Mirror { action } => {
                let mirror_mgr = MirrorManager::new(".")?;
                match action {
                    MirrorCommands::AddPrimary { platform, account_type, account, repo, protocol } => {
                        let acc_type = parse_account_type(account_type)?;
                        let proto = parse_protocol(protocol.as_ref());
                        mirror_mgr.add_mirror(platform, acc_type, account, repo, proto, true)?;
                        println!("✅ Primary mirror added: : {}/{} on {}", account, repo, platform);
                    }
                    MirrorCommands::AddReplica { platform, account_type, account, repo, protocol } => {
                        let acc_type = parse_account_type(account_type)?;
                        let proto = parse_protocol(protocol.as_ref());
                        mirror_mgr.add_mirror(platform, acc_type, account, repo, proto, false)?;
                        println!("✅ Replica mirror added: {}/{} on {}", account, repo, platform);
                    }
                    MirrorCommands::List => {
                        mirror_mgr.list_mirrors()?;
                    }
                    MirrorCommands::Sync { force } => {
                        mirror_mgr.sync_all(*force)?;
                    }
                    MirrorCommands::SetPrimary { platform, account } => {
                        mirror_mgr.set_primary(platform, account)?;
                        println!("✅ Set as primary: {}/{}", platform, account);
                    }
                    MirrorCommands::Remove { platform, account } => {
                        mirror_mgr.remove_mirror_by_account(platform, account)?;
                        println!("✅ Mirror removed: {}/{}", platform, account);
                    }
                    MirrorCommands::Autofetch { enable, disable, interval, status } => {
                        if *status {
                            mirror_mgr.show_autofetch_status()?;
                        } else if *enable {
                            let interval_minutes = if let Some(interval_str) = interval {
                                Some(parse_duration(interval_str)?)
                            } else {
                                None
                            };
                            mirror_mgr.configure_autofetch(true, interval_minutes)?;
                        } else if *disable {
                            mirror_mgr.configure_autofetch(false, None)?;
                        } else {
                            mirror_mgr.show_autofetch_status()?;
                        }
                    }
                }
            }

            Commands::SshCheck => {
                println!("🔐 SSH Configuration Check\n");
                
                if SshHelper::has_ssh_keys() {
                    println!("✅ SSH keys found!\n");
                    
                    let keys = SshHelper::list_keys();
                    if !keys.is_empty() {
                        println!("Available keys:");
                        for key in &keys {
                            println!("  • {}", key);
                        }
                    }
                    
                    println!("\n💡 Recommendation: Use SSH protocol (default)");
                } else {
                    println!("❌ No SSH keys found");
                    println!("\n💡 To set up SSH keys:");
                    println!("   1. Generate a new key:");
                    println!("      ssh-keygen -t ed25519 -C \"your_email@example.com\"");
                    println!("   2. Start the SSH agent:");
                    println!("      eval \"$(ssh-agent -s)\"");
                    println!("   3. Add your key:");
                    println!("      ssh-add ~/.ssh/id_ed25519");
                    println!("   4. Copy your public key:");
                    println!("      cat ~/.ssh/id_ed25519.pub");
                    println!("   5. Add it to your Git hosting service");
                }
            }

            Commands::Config { action } => {
                match action {
                    ConfigCommands::Set { key, value, local } => {
                        if *local {
                            let mut config = ToriiConfig::load_local(".")?;
                            config.set(key, value)?;
                            config.save_local(".")?;
                            println!("✅ Local config updated: {} = {}", key, value);
                        } else {
                            let mut config = ToriiConfig::load_global()?;
                            config.set(key, value)?;
                            config.save_global()?;
                            println!("✅ Global config updated: {} = {}", key, value);
                        }
                    }
                    ConfigCommands::Get { key, local } => {
                        let config = if *local {
                            ToriiConfig::load_local(".")?
                        } else {
                            ToriiConfig::load_global()?
                        };
                        
                        if let Some(value) = config.get(key) {
                            println!("{}", value);
                        } else {
                            println!("❌ Config key not found: {}", key);
                        }
                    }
                    ConfigCommands::List { local } => {
                        let config = if *local {
                            ToriiConfig::load_local(".")?
                        } else {
                            ToriiConfig::load_global()?
                        };
                        
                        let scope = if *local { "Local" } else { "Global" };
                        println!("⚙️  {} Configuration:\n", scope);
                        
                        for (key, value) in config.list() {
                            println!("  {} = {}", key, value);
                        }
                    }
                    ConfigCommands::Edit { local } => {
                        let config_path = if *local {
                            std::path::PathBuf::from(".").join(".torii").join("config.toml")
                        } else {
                            dirs::config_dir()
                                .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?
                                .join("torii")
                                .join("config.toml")
                        };
                        
                        // Ensure config exists
                        if *local {
                            let config = ToriiConfig::load_local(".")?;
                            config.save_local(".")?;
                        } else {
                            let config = ToriiConfig::load_global()?;
                            config.save_global()?;
                        }
                        
                        // Get editor
                        let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
                        
                        // Open editor
                        let status = std::process::Command::new(&editor)
                            .arg(&config_path)
                            .status()?;
                        
                        if status.success() {
                            println!("✅ Configuration edited");
                        } else {
                            println!("❌ Editor exited with error");
                        }
                    }
                    ConfigCommands::Reset { local } => {
                        let config = ToriiConfig::default();
                        
                        if *local {
                            config.save_local(".")?;
                            println!("✅ Local configuration reset to defaults");
                        } else {
                            config.save_global()?;
                            println!("✅ Global configuration reset to defaults");
                        }
                    }
                }
            }

            Commands::Remote { action } => {
                match action {
                    RemoteCommands::Create { platform, name, description, public, private: _, push } => {
                        let client = get_platform_client(platform)?;
                        
                        let visibility = if *public {
                            Visibility::Public
                        } else {
                            Visibility::Private
                        };
                        
                        println!("🚀 Creating repository '{}' on {}...", name, platform);
                        let repo = client.create_repo(name, description.as_deref(), visibility)?;
                        
                        println!("✅ Repository created successfully!");
                        println!("   URL: {}", repo.url);
                        println!("   SSH: {}", repo.ssh_url);
                        
                        if *push {
                            println!("\n📤 Pushing to remote...");
                            let git_repo = GitRepo::open(".")?;
                            // Add remote via git2
                            let _ = git_repo.repository().remote("origin", &repo.ssh_url);
                            git_repo.push(false)?;
                            println!("✅ Pushed to remote");
                        }
                    }
                    RemoteCommands::Delete { platform, owner, repo, yes } => {
                        if !yes {
                            println!("⚠️  Are you sure you want to delete {}/{}? This cannot be undone!", owner, repo);
                            println!("   Run with --yes to confirm");
                            return Ok(());
                        }
                        
                        let client = get_platform_client(platform)?;
                        println!("🗑️  Deleting repository {}/{}...", owner, repo);
                        client.delete_repo(owner, repo)?;
                    }
                    RemoteCommands::Visibility { platform, owner, repo, public, private } => {
                        let client = get_platform_client(platform)?;
                        
                        let visibility = if *public {
                            Visibility::Public
                        } else if *private {
                            Visibility::Private
                        } else {
                            println!("❌ Specify --public or --private");
                            return Ok(());
                        };
                        
                        println!("🔒 Changing visibility of {}/{} to {:?}...", owner, repo, visibility);
                        client.set_visibility(owner, repo, visibility)?;
                        println!("✅ Visibility updated");
                    }
                    RemoteCommands::Configure { 
                        platform, owner, repo, description, homepage, default_branch,
                        enable_issues, disable_issues, enable_wiki, disable_wiki,
                        enable_projects, disable_projects 
                    } => {
                        let client = get_platform_client(platform)?;
                        
                        // Build settings
                        let mut settings = RepoSettings::default();
                        settings.description = description.clone();
                        settings.homepage = homepage.clone();
                        settings.default_branch = default_branch.clone();
                        
                        // Build features
                        let mut features = RepoFeatures::default();
                        if *enable_issues { features.issues = Some(true); }
                        if *disable_issues { features.issues = Some(false); }
                        if *enable_wiki { features.wiki = Some(true); }
                        if *disable_wiki { features.wiki = Some(false); }
                        if *enable_projects { features.projects = Some(true); }
                        if *disable_projects { features.projects = Some(false); }
                        
                        println!("⚙️  Configuring repository {}/{}...", owner, repo);
                        
                        // Update settings if any
                        if settings.description.is_some() || settings.homepage.is_some() || settings.default_branch.is_some() {
                            client.update_repo(owner, repo, settings)?;
                        }
                        
                        // Update features if any
                        if features.issues.is_some() || features.wiki.is_some() || features.projects.is_some() {
                            client.configure_features(owner, repo, features)?;
                        }
                        
                        println!("✅ Repository configured");
                    }
                    RemoteCommands::Info { platform, owner, repo } => {
                        let client = get_platform_client(platform)?;
                        println!("📊 Fetching repository information...");
                        let repo_info = client.get_repo(owner, repo)?;
                        
                        println!("\n📦 Repository: {}", repo_info.name);
                        if let Some(desc) = &repo_info.description {
                            println!("   Description: {}", desc);
                        }
                        println!("   Visibility: {:?}", repo_info.visibility);
                        println!("   Default Branch: {}", repo_info.default_branch);
                        println!("   URL: {}", repo_info.url);
                        println!("   SSH: {}", repo_info.ssh_url);
                    }
                    RemoteCommands::List { platform } => {
                        let client = get_platform_client(platform)?;
                        println!("📋 Fetching repositories from {}...", platform);
                        let repos = client.list_repos()?;
                        
                        if repos.is_empty() {
                            println!("No repositories found");
                        } else {
                            println!("\n📦 Repositories ({}):\n", repos.len());
                            for repo in repos {
                                println!("  • {} - {:?}", repo.name, repo.visibility);
                                if let Some(desc) = &repo.description {
                                    println!("    {}", desc);
                                }
                            }
                        }
                    }
                }
            }

            Commands::Repo { 
                name, platforms, create, delete, public, private: _,
                description, push, yes, owner
            } => {
                use crate::remote::{get_platform_client, Visibility};
                
                if platforms.is_empty() {
                    println!("❌ No platforms specified. Use --platforms github,gitlab,codeberg");
                    return Ok(());
                }
                
                // Validate operation
                if !create && !delete {
                    println!("❌ Specify an operation: --create or --delete");
                    return Ok(());
                }
                
                if *create && *delete {
                    println!("❌ Cannot create and delete at the same time");
                    return Ok(());
                }
                
                let visibility = if *public {
                    Visibility::Public
                } else {
                    Visibility::Private
                };
                
                println!("🌐 Multi-platform operation on {} platforms", platforms.len());
                println!("   Repository: {}", name);
                println!("   Platforms: {}", platforms.join(", "));
                
                if *delete && !yes {
                    println!("\n⚠️  WARNING: This will DELETE '{}' from {} platforms!", name, platforms.len());
                    println!("   This action CANNOT be undone!");
                    println!("   Run with --yes to confirm");
                    return Ok(());
                }
                
                let mut results = Vec::new();
                
                for platform in platforms {
                    print!("\n📦 {} - ", platform);
                    
                    match get_platform_client(platform) {
                        Ok(client) => {
                            if *create {
                                print!("Creating... ");
                                match client.create_repo(name, description.as_deref(), visibility.clone()) {
                                    Ok(repo) => {
                                        println!("✅ Created");
                                        println!("   URL: {}", repo.url);
                                        results.push((platform.clone(), true, None));
                                    }
                                    Err(e) => {
                                        println!("❌ Failed: {}", e);
                                        results.push((platform.clone(), false, Some(e.to_string())));
                                    }
                                }
                            } else if *delete {
                                print!("Deleting... ");
                                let owner_name = owner.as_ref()
                                    .map(|s| s.as_str())
                                    .unwrap_or("user");
                                
                                match client.delete_repo(owner_name, name) {
                                    Ok(_) => {
                                        println!("✅ Deleted");
                                        results.push((platform.clone(), true, None));
                                    }
                                    Err(e) => {
                                        println!("❌ Failed: {}", e);
                                        results.push((platform.clone(), false, Some(e.to_string())));
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            println!("❌ Platform error: {}", e);
                            results.push((platform.clone(), false, Some(e.to_string())));
                        }
                    }
                }
                
                // Summary
                let successful = results.iter().filter(|(_, success, _)| *success).count();
                let failed = results.len() - successful;
                
                println!("\n📊 Summary:");
                println!("   ✅ Successful: {}/{}", successful, results.len());
                if failed > 0 {
                    println!("   ❌ Failed: {}", failed);
                    println!("\n   Failed platforms:");
                    for (platform, success, error) in results.iter() {
                        if !success {
                            println!("     • {}: {}", platform, error.as_ref().unwrap_or(&"Unknown error".to_string()));
                        }
                    }
                }
                
                // Push if requested and created successfully
                if *create && *push && successful > 0 {
                    println!("\n📤 Pushing to remote...");
                    let git_repo = GitRepo::open(".")?;
                    
                    // Add remotes for successful platforms via git2
                    for (platform, success, _) in results.iter() {
                        if *success {
                            let url = format!("git@{}:{}/{}.git", platform, owner.as_ref().unwrap_or(&"user".to_string()), name);
                            let _ = git_repo.repository().remote(platform, &url);
                        }
                    }
                    
                    git_repo.push(false)?;
                    println!("✅ Pushed to remotes");
                }
            }

            Commands::Ls { path } => {
                let repo = GitRepo::open(".")?;
                repo.ls(path.as_deref())?;
            }

            Commands::Show { object, blame, lines } => {
                let repo = GitRepo::open(".")?;
                if *blame {
                    let file = object.as_deref().ok_or_else(|| anyhow::anyhow!("File path required for --blame"))?;
                    repo.blame(file, lines.as_deref())?;
                } else {
                    repo.show(object.as_deref())?;
                }
            }

            Commands::History { action } => {
                let repo = GitRepo::open(".")?;
                match action {
                    HistoryCommands::Rewrite { start, end } => {
                        repo.rewrite_history(start, end)?;
                        println!("✅ History rewritten successfully");
                    }
                    HistoryCommands::Clean => {
                        repo.clean_history()?;
                        println!("✅ Repository cleaned");
                    }
                    HistoryCommands::VerifyRemote => {
                        repo.verify_remote()?;
                    }
                    HistoryCommands::Reflog { count } => {
                        repo.show_reflog(*count)?;
                    }
                    HistoryCommands::RemoveFile { file } => {
                        repo.remove_file_from_history(file)?;
                    }
                    HistoryCommands::CherryPick { commit, r#continue, abort } => {
                        if *r#continue {
                            repo.cherry_pick_continue()?;
                        } else if *abort {
                            repo.cherry_pick_abort()?;
                        } else {
                            let hash = commit.as_deref().ok_or_else(|| anyhow::anyhow!("Commit hash required: torii history cherry-pick <hash>"))?;
                            repo.cherry_pick(hash)?;
                        }
                    }
                    HistoryCommands::Blame { file, lines } => {
                        repo.blame(file, lines.as_deref())?;
                    }
                    HistoryCommands::Scan { history } => {
                        let repo_path = std::path::Path::new(".");
                        if *history {
                            println!("🔍 Scanning full git history for sensitive data...\n");
                            let results = scanner::scan_history(repo_path)?;
                            if results.is_empty() {
                                println!("✅ No sensitive data found in history.");
                            } else {
                                println!("⚠️  Found sensitive data in {} commit(s):\n", results.len());
                                for (commit, findings) in &results {
                                    println!("  📌 {}", commit);
                                    for f in findings {
                                        println!("     {}:{} — {}", f.file, f.line, f.pattern_name);
                                        println!("     {}", f.preview);
                                    }
                                    println!();
                                }
                                println!("💡 To clean history: torii history rebase <base> --todo-file <plan>");
                            }
                        } else {
                            println!("🔍 Scanning staged files for sensitive data...\n");
                            let findings = scanner::scan_staged(repo_path)?;
                            if findings.is_empty() {
                                println!("✅ No sensitive data detected in staged files.");
                            } else {
                                println!("⚠️  Found {} issue(s):\n", findings.len());
                                for f in &findings {
                                    println!("  {}:{} — {}", f.file, f.line, f.pattern_name);
                                    println!("  {}\n", f.preview);
                                }
                                println!("💡 Tip: use .env.example for placeholder values.");
                            }
                        }
                    }
                    HistoryCommands::Rebase { target, interactive, todo_file, r#continue, abort, skip } => {
                        if *r#continue {
                            repo.rebase_continue()?;
                        } else if *abort {
                            repo.rebase_abort()?;
                        } else if *skip {
                            repo.rebase_skip()?;
                        } else if let Some(todo) = todo_file {
                            let base = target.as_deref().ok_or_else(|| anyhow::anyhow!("Target required: torii history rebase <base> --todo-file plan.txt"))?;
                            repo.rebase_with_todo(base, todo)?;
                        } else if *interactive {
                            let base = target.as_deref().ok_or_else(|| anyhow::anyhow!("Target required: torii history rebase HEAD~3 --interactive"))?;
                            repo.rebase_interactive(base)?;
                        } else if let Some(base) = target {
                            repo.rebase_branch(base)?;
                            println!("✅ Rebased onto: {}", base);
                        } else {
                            anyhow::bail!("Specify a target or use --interactive / --todo-file / --continue / --abort / --skip");
                        }
                    }
                }
            }

            Commands::Workspace { action } => {
                use crate::workspace::WorkspaceManager;
                match action {
                    WorkspaceCommands::Add { workspace, path } => {
                        WorkspaceManager::add(workspace, path)?;
                    }
                    WorkspaceCommands::Remove { workspace, path } => {
                        WorkspaceManager::remove(workspace, path)?;
                    }
                    WorkspaceCommands::Delete { workspace } => {
                        WorkspaceManager::delete(workspace)?;
                    }
                    WorkspaceCommands::List => {
                        WorkspaceManager::list()?;
                    }
                    WorkspaceCommands::Status { workspace } => {
                        WorkspaceManager::status(workspace)?;
                    }
                    WorkspaceCommands::Save { workspace, message, all } => {
                        WorkspaceManager::save(workspace, message, *all)?;
                    }
                    WorkspaceCommands::Sync { workspace, force } => {
                        WorkspaceManager::sync(workspace, *force)?;
                    }
                }
            }

            Commands::Tui => {
                let current = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
                if git2::Repository::discover(&current).is_ok() {
                    // Estamos dentro de un repo — abre directamente
                    crate::tui::run()?;
                } else {
                    // No hay repo — lanza el picker
                    use crate::tui::picker::{run_picker, save_workspace, PickerResult};
                    match run_picker(&current)? {
                        PickerResult::Cancelled => {}
                        PickerResult::SingleRepo(path) => {
                            std::env::set_current_dir(&path)?;
                            crate::tui::run()?;
                        }
                        PickerResult::Workspace { name, repos } => {
                            save_workspace(&name, &repos)?;
                            // Abre TUI en el primer repo del workspace, vista Workspace
                            if let Some(first) = repos.first() {
                                std::env::set_current_dir(first)?;
                            }
                            crate::tui::run_with_view(crate::tui::app::View::Workspace)?;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
