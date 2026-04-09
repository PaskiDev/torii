use clap::{Parser, Subcommand};
use anyhow::Result;
use crate::core::GitRepo;
use crate::snapshot::SnapshotManager;
use crate::mirror::{MirrorManager, AccountType, Protocol};
use crate::ssh::SshHelper;

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
#[command(version, about = "A modern git client with simplified commands", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new repository
    Init {
        /// Path to initialize (defaults to current directory)
        #[arg(short, long)]
        path: Option<String>,
    },

    /// Save current work (simplified commit)
    Save {
        /// Commit message
        #[arg(short, long)]
        message: String,

        /// Add all changes before committing
        #[arg(short, long)]
        all: bool,

        /// Amend the previous commit
        #[arg(long)]
        amend: bool,
    },

    /// Sync with remote (simplified push/pull)
    Sync {
        /// Force push
        #[arg(short, long)]
        force: bool,
    },

    /// Show repository status
    Status,

    /// Show commit history
    Log {
        /// Number of commits to show
        #[arg(short = 'n', long)]
        count: Option<usize>,

        /// Show as one line per commit
        #[arg(long)]
        oneline: bool,

        /// Show graph
        #[arg(long)]
        graph: bool,
    },

    /// Show changes
    Diff {
        /// Show staged changes
        #[arg(long)]
        staged: bool,

        /// Show last commit
        #[arg(long)]
        last: bool,
    },

    /// Manage branches
    Branch {
        /// Branch name to create
        name: Option<String>,

        /// Delete branch
        #[arg(short = 'd', long)]
        delete: bool,

        /// List all branches
        #[arg(short = 'a', long)]
        all: bool,
    },

    /// Switch to a branch
    Switch {
        /// Branch name
        branch: String,

        /// Create new branch
        #[arg(short = 'c', long)]
        create: bool,
    },

    /// Clone a repository
    Clone {
        /// Repository URL or platform shorthand (e.g., github user/repo)
        source: String,

        /// Additional arguments for platform shorthand
        args: Vec<String>,

        /// Target directory
        #[arg(short = 'd', long)]
        directory: Option<String>,
    },

    /// Integrate changes from another branch (smart merge/rebase)
    Integrate {
        /// Branch to integrate
        branch: String,

        /// Force merge (even if rebase is recommended)
        #[arg(long)]
        merge: bool,

        /// Force rebase (even if merge is recommended)
        #[arg(long)]
        rebase: bool,

        /// Show preview without executing
        #[arg(long)]
        preview: bool,
    },

    /// Tag management
    Tag {
        #[command(subcommand)]
        action: TagCommands,
    },

    /// Apply a commit to current branch
    CherryPick {
        /// Commit hash to cherry-pick
        commit: String,

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

    /// Snapshot management
    Snapshot {
        #[command(subcommand)]
        action: SnapshotCommands,
    },

    /// Mirror management (multi-platform sync)
    Mirror {
        #[command(subcommand)]
        action: MirrorCommands,
    },

    /// Check SSH configuration and get setup help
    SshCheck,
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
}

#[derive(Subcommand)]
enum MirrorCommands {
    /// Add master mirror (main repository)
    AddMaster {
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

    /// Add slave mirror (will sync from master)
    AddSlave {
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

    /// Sync to all slave mirrors
    Sync {
        /// Force sync
        #[arg(short, long)]
        force: bool,
    },

    /// Set a mirror as master
    SetMaster {
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
                println!("✅ Initialized repository at {}", repo_path);
            }

            Commands::Save { message, all, amend } => {
                let repo = GitRepo::open(".")?;
                
                if *all {
                    repo.add_all()?;
                }
                
                if *amend {
                    repo.commit_amend(message)?;
                    println!("✅ Commit amended: {}", message);
                } else {
                    repo.commit(message)?;
                    println!("✅ Changes saved: {}", message);
                }
            }

            Commands::Sync { force } => {
                let repo = GitRepo::open(".")?;
                repo.pull()?;
                repo.push(*force)?;
                println!("✅ Synced with remote");
            }

            Commands::Status => {
                let repo = GitRepo::open(".")?;
                repo.status()?;
            }

            Commands::Log { count, oneline, graph } => {
                let repo = GitRepo::open(".")?;
                repo.log(*count, *oneline, *graph)?;
            }

            Commands::Diff { staged, last } => {
                let repo = GitRepo::open(".")?;
                repo.diff(*staged, *last)?;
            }

            Commands::Branch { name, delete, all } => {
                let repo = GitRepo::open(".")?;
                
                if *delete {
                    if let Some(branch_name) = name {
                        repo.delete_branch(branch_name)?;
                        println!("✅ Deleted branch: {}", branch_name);
                    } else {
                        anyhow::bail!("Branch name required for deletion");
                    }
                } else if let Some(branch_name) = name {
                    repo.create_branch(branch_name)?;
                    println!("✅ Created branch: {}", branch_name);
                } else {
                    repo.list_branches(*all)?;
                }
            }

            Commands::Switch { branch, create } => {
                let repo = GitRepo::open(".")?;
                
                if *create {
                    repo.create_branch(branch)?;
                    println!("✅ Created branch: {}", branch);
                }
                
                repo.switch_branch(branch)?;
                println!("✅ Switched to branch: {}", branch);
            }

            Commands::Clone { source, args, directory } => {
                if args.is_empty() && !source.starts_with("http") && !source.starts_with("git@") {
                    anyhow::bail!("Use: torii clone <platform> <user/repo> or provide full URL");
                }
                
                let url = if !args.is_empty() {
                    let platform = source;
                    let user_repo = &args[0];
                    let protocol = if SshHelper::has_ssh_keys() { "ssh" } else { "https" };
                    
                    match platform.as_str() {
                        "github" => {
                            if protocol == "ssh" {
                                format!("git@github.com:{}.git", user_repo)
                            } else {
                                format!("https://github.com/{}.git", user_repo)
                            }
                        }
                        "gitlab" => {
                            if protocol == "ssh" {
                                format!("git@gitlab.com:{}.git", user_repo)
                            } else {
                                format!("https://gitlab.com/{}.git", user_repo)
                            }
                        }
                        "codeberg" => {
                            if protocol == "ssh" {
                                format!("git@codeberg.org:{}.git", user_repo)
                            } else {
                                format!("https://codeberg.org/{}.git", user_repo)
                            }
                        }
                        "bitbucket" => {
                            if protocol == "ssh" {
                                format!("git@bitbucket.org:{}.git", user_repo)
                            } else {
                                format!("https://bitbucket.org/{}.git", user_repo)
                            }
                        }
                        _ => anyhow::bail!("Unknown platform: {}", platform),
                    }
                } else {
                    source.clone()
                };
                
                let target_dir = directory.as_deref();
                GitRepo::clone_repo(&url, target_dir)?;
                
                let dir_name = target_dir.unwrap_or_else(|| {
                    url.split('/').last().unwrap_or("repo").trim_end_matches(".git")
                });
                println!("✅ Cloned repository to: {}", dir_name);
            }

            Commands::Integrate { branch, merge, rebase, preview } => {
                let repo = GitRepo::open(".")?;
                repo.integrate(branch, *merge, *rebase, *preview)?;
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
                }
            }

            Commands::CherryPick { commit, r#continue, abort } => {
                let repo = GitRepo::open(".")?;
                if *r#continue {
                    repo.cherry_pick_continue()?;
                } else if *abort {
                    repo.cherry_pick_abort()?;
                } else {
                    repo.cherry_pick(commit)?;
                }
            }

            Commands::Blame { file, lines } => {
                let repo = GitRepo::open(".")?;
                repo.blame(file, lines.as_deref())?;
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
                    MirrorCommands::AddMaster { platform, account_type, account, repo, protocol } => {
                        let acc_type = parse_account_type(account_type)?;
                        let proto = parse_protocol(protocol.as_ref());
                        mirror_mgr.add_mirror(platform, acc_type, account, repo, proto, true)?;
                        println!("✅ Master mirror added: {}/{} on {}", account, repo, platform);
                    }
                    MirrorCommands::AddSlave { platform, account_type, account, repo, protocol } => {
                        let acc_type = parse_account_type(account_type)?;
                        let proto = parse_protocol(protocol.as_ref());
                        mirror_mgr.add_mirror(platform, acc_type, account, repo, proto, false)?;
                        println!("✅ Slave mirror added: {}/{} on {}", account, repo, platform);
                    }
                    MirrorCommands::List => {
                        mirror_mgr.list_mirrors()?;
                    }
                    MirrorCommands::Sync { force } => {
                        mirror_mgr.sync_all(*force)?;
                    }
                    MirrorCommands::SetMaster { platform, account } => {
                        mirror_mgr.set_master(platform, account)?;
                        println!("✅ Set {}/{} as master", platform, account);
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
                println!("🔍 Checking SSH configuration...\n");
                
                if SshHelper::has_ssh_keys() {
                    println!("✅ SSH keys found!\n");
                    
                    let keys = SshHelper::list_keys();
                    println!("Available keys:");
                    for key in &keys {
                        println!("  • {}", key);
                        if let Ok(pub_key) = SshHelper::get_public_key(key) {
                            if !pub_key.is_empty() {
                                let preview = pub_key.chars().take(60).collect::<String>();
                                println!("    {}", preview);
                                if pub_key.len() > 60 {
                                    println!("    ...");
                                }
                            }
                        }
                    }
                    
                    println!("\n💡 Recommendation: Use SSH protocol (default)");
            }
        }

        Ok(())
    }
