use std::path::{Path, PathBuf};
use std::fs;
use serde::{Deserialize, Serialize};
use crate::error::{Result, ToriiError};
use crate::core::GitRepo;
use crate::duration::format_duration;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum MirrorType {
    Master,
    Slave,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum AccountType {
    User,
    Organization,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Protocol {
    SSH,
    HTTPS,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Mirror {
    pub name: String,
    pub platform: String,
    pub account_type: AccountType,
    pub account_name: String,
    pub repo_name: String,
    pub url: String,
    pub protocol: Protocol,
    pub mirror_type: MirrorType,
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct MirrorConfig {
    mirrors: Vec<Mirror>,
    #[serde(default)]
    autofetch_enabled: bool,
    #[serde(default = "default_autofetch_interval")]
    autofetch_interval_minutes: u32,
}

fn default_autofetch_interval() -> u32 {
    30 // Default: 30 minutes
}

impl Mirror {
    /// Generate URL based on platform, account info, and protocol
    pub fn generate_url(
        platform: &str, 
        _account_type: &AccountType, 
        account_name: &str, 
        repo_name: &str,
        protocol: &Protocol,
    ) -> String {
        match protocol {
            Protocol::SSH => {
                match platform.to_lowercase().as_str() {
                    "github" => format!("git@github.com:{}/{}.git", account_name, repo_name),
                    "gitlab" => format!("git@gitlab.com:{}/{}.git", account_name, repo_name),
                    "bitbucket" => format!("git@bitbucket.org:{}/{}.git", account_name, repo_name),
                    "codeberg" => format!("git@codeberg.org:{}/{}.git", account_name, repo_name),
                    "gitea" => format!("git@gitea.com:{}/{}.git", account_name, repo_name),
                    "forgejo" => format!("git@codeberg.org:{}/{}.git", account_name, repo_name),
                    "sourcehut" | "srht" => format!("git@git.sr.ht:~{}/{}", account_name, repo_name),
                    "sourceforge" => format!("git@git.code.sf.net:p/{}/{}", account_name, repo_name),
                    _ => format!("git@{}:{}/{}.git", platform, account_name, repo_name),
                }
            }
            Protocol::HTTPS => {
                match platform.to_lowercase().as_str() {
                    "github" => format!("https://github.com/{}/{}.git", account_name, repo_name),
                    "gitlab" => format!("https://gitlab.com/{}/{}.git", account_name, repo_name),
                    "bitbucket" => format!("https://bitbucket.org/{}/{}.git", account_name, repo_name),
                    "codeberg" => format!("https://codeberg.org/{}/{}.git", account_name, repo_name),
                    "gitea" => format!("https://gitea.com/{}/{}.git", account_name, repo_name),
                    "forgejo" => format!("https://codeberg.org/{}/{}.git", account_name, repo_name),
                    "sourcehut" | "srht" => format!("https://git.sr.ht/~{}/{}", account_name, repo_name),
                    "sourceforge" => format!("https://git.code.sf.net/p/{}/{}", account_name, repo_name),
                    _ => format!("https://{}/{}/{}.git", platform, account_name, repo_name),
                }
            }
        }
    }
    
    /// Get display name for the mirror
    #[allow(dead_code)]
    pub fn display_name(&self) -> String {
        format!("{}/{}", self.account_name, self.repo_name)
    }
}

pub struct MirrorManager {
    repo_path: PathBuf,
    config_path: PathBuf,
}

impl MirrorManager {
    pub fn new<P: AsRef<Path>>(repo_path: P) -> Result<Self> {
        let repo_path = repo_path.as_ref().to_path_buf();
        let torii_dir = repo_path.join(".torii");
        fs::create_dir_all(&torii_dir)?;
        
        let config_path = torii_dir.join("mirrors.json");

        Ok(Self {
            repo_path,
            config_path,
        })
    }

    /// Load mirror configuration
    fn load_config(&self) -> Result<MirrorConfig> {
        if !self.config_path.exists() {
            return Ok(MirrorConfig { 
                mirrors: vec![],
                autofetch_enabled: false,
                autofetch_interval_minutes: 30,
            });
        }

        let config_str = fs::read_to_string(&self.config_path)?;
        let config: MirrorConfig = serde_json::from_str(&config_str)?;
        
        Ok(config)
    }

    /// Save mirror configuration
    fn save_config(&self, config: &MirrorConfig) -> Result<()> {
        let config_str = serde_json::to_string_pretty(config)?;
        fs::write(&self.config_path, config_str)?;
        Ok(())
    }

    /// Add a new mirror with simplified interface
    pub fn add_mirror(
        &self,
        platform: &str,
        account_type: AccountType,
        account_name: &str,
        repo_name: &str,
        protocol: Protocol,
        is_master: bool,
    ) -> Result<()> {
        let mut config = self.load_config()?;
        
        // Check if master already exists
        if is_master && config.mirrors.iter().any(|m| m.mirror_type == MirrorType::Master) {
            return Err(ToriiError::Mirror(
                "A master mirror already exists. Use 'torii mirror set-master' to change it.".to_string()
            ));
        }
        
        // Generate URL automatically
        let url = Mirror::generate_url(platform, &account_type, account_name, repo_name, &protocol);
        
        // Generate remote name
        let remote_name = if is_master {
            "origin".to_string()
        } else {
            format!("{}-{}", platform, account_name)
        };

        let mirror = Mirror {
            name: remote_name.clone(),
            platform: platform.to_string(),
            account_type,
            account_name: account_name.to_string(),
            repo_name: repo_name.to_string(),
            url: url.clone(),
            protocol,
            mirror_type: if is_master { MirrorType::Master } else { MirrorType::Slave },
            enabled: true,
        };

        config.mirrors.push(mirror);
        self.save_config(&config)?;

        let repo = GitRepo::open(&self.repo_path)?;
        self.add_git_remote(&repo, &remote_name, &url)?;

        Ok(())
    }
    
    /// Set a mirror as master
    pub fn set_master(&self, platform: &str, account_name: &str) -> Result<()> {
        let mut config = self.load_config()?;
        
        // Find the mirror
        let mirror_index = config.mirrors.iter().position(|m| {
            m.platform == platform && m.account_name == account_name
        }).ok_or_else(|| ToriiError::Mirror("Mirror not found".to_string()))?;
        
        // Set all to slave
        for mirror in &mut config.mirrors {
            mirror.mirror_type = MirrorType::Slave;
        }
        
        // Set selected as master
        config.mirrors[mirror_index].mirror_type = MirrorType::Master;
        
        self.save_config(&config)?;
        Ok(())
    }

    /// Add git remote
    fn add_git_remote(&self, repo: &GitRepo, name: &str, url: &str) -> Result<()> {
        repo.repository().remote(name, url)?;
        Ok(())
    }

    /// List all mirrors
    pub fn list_mirrors(&self) -> Result<()> {
        let config = self.load_config()?;

        if config.mirrors.is_empty() {
            println!("No mirrors configured");
            println!();
            println!("💡 Add a master mirror first:");
            println!("   torii mirror add-master <platform> <user|org> <account> <repo>");
            return Ok(());
        }

        println!("🪞 Configured Mirrors:");
        println!();

        // Show master first
        for mirror in config.mirrors.iter().filter(|m| m.mirror_type == MirrorType::Master) {
            let status = if mirror.enabled { "✅" } else { "❌" };
            let account_type = match mirror.account_type {
                AccountType::User => "👤",
                AccountType::Organization => "🏢",
            };
            let protocol_icon = match mirror.protocol {
                Protocol::SSH => "🔑",
                Protocol::HTTPS => "🌐",
            };
            println!("  {} 👑 MASTER - {} {} {} {}/{}", 
                status, 
                protocol_icon,
                account_type,
                mirror.platform,
                mirror.account_name,
                mirror.repo_name
            );
            println!("     {}", mirror.url);
            println!();
        }

        // Show slaves
        let slaves: Vec<_> = config.mirrors.iter()
            .filter(|m| m.mirror_type == MirrorType::Slave)
            .collect();
            
        if !slaves.is_empty() {
            println!("  Slave Mirrors:");
            for mirror in slaves {
                let status = if mirror.enabled { "✅" } else { "❌" };
                let account_type = match mirror.account_type {
                    AccountType::User => "👤",
                    AccountType::Organization => "🏢",
                };
                let protocol_icon = match mirror.protocol {
                    Protocol::SSH => "🔑",
                    Protocol::HTTPS => "🌐",
                };
                println!("    {} {} {} {} {}/{}", 
                    status,
                    protocol_icon,
                    account_type,
                    mirror.platform,
                    mirror.account_name,
                    mirror.repo_name
                );
                println!("       {}", mirror.url);
            }
        }

        Ok(())
    }

    /// Sync to all slave mirrors (push from master)
    pub fn sync_all(&self, force: bool) -> Result<()> {
        let config = self.load_config()?;
        let repo = GitRepo::open(&self.repo_path)?;

        // Find master mirror
        let master = config.mirrors.iter()
            .find(|m| m.mirror_type == MirrorType::Master);
        
        if master.is_none() {
            println!("⚠️  No master mirror configured. Add one with:");
            println!("   torii mirror add-master <platform> <user|org> <account> <repo>");
            return Ok(());
        }

        // Get slave mirrors
        let slaves: Vec<_> = config.mirrors.iter()
            .filter(|m| m.mirror_type == MirrorType::Slave && m.enabled)
            .collect();

        if slaves.is_empty() {
            println!("ℹ️  No slave mirrors configured. Add one with:");
            println!("   torii mirror add-slave <platform> <user|org> <account> <repo>");
            return Ok(());
        }

        println!("📤 Syncing from master to {} slave mirror(s)...\n", slaves.len());

        let mut success_count = 0;
        let mut fail_count = 0;

        for mirror in slaves {
            println!("🔄 Syncing to {} {}/{} ...", 
                mirror.platform, 
                mirror.account_name, 
                mirror.repo_name
            );
            
            match self.sync_to_mirror(&repo, mirror, force) {
                Ok(_) => {
                    println!("  ✅ Synced successfully\n");
                    success_count += 1;
                }
                Err(e) => {
                    eprintln!("  ❌ Failed: {}\n", e);
                    fail_count += 1;
                    if !force {
                        return Err(e);
                    }
                }
            }
        }

        println!("📊 Summary: {} succeeded, {} failed", success_count, fail_count);
        Ok(())
    }

    /// Sync to a specific mirror
    fn sync_to_mirror(&self, repo: &GitRepo, mirror: &Mirror, force: bool) -> Result<()> {
        let mut remote = repo.repository().find_remote(&mirror.name)?;
        let branch = repo.get_current_branch()?;
        
        let refspec = if force {
            format!("+refs/heads/{}:refs/heads/{}", branch, branch)
        } else {
            format!("refs/heads/{}:refs/heads/{}", branch, branch)
        };

        // Setup callbacks for SSH authentication — try explicit keys then agent
        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(|_url, username_from_url, _allowed_types| {
            let username = username_from_url.unwrap_or("git");
            let home = std::env::var("HOME").unwrap_or_default();
            // Try ed25519 first, then rsa, then agent
            let ed25519 = std::path::Path::new(&home).join(".ssh/id_ed25519");
            let rsa = std::path::Path::new(&home).join(".ssh/id_rsa");
            if ed25519.exists() {
                git2::Cred::ssh_key(username, None, &ed25519, None)
            } else if rsa.exists() {
                git2::Cred::ssh_key(username, None, &rsa, None)
            } else {
                git2::Cred::ssh_key_from_agent(username)
            }
        });

        let mut push_options = git2::PushOptions::new();
        push_options.remote_callbacks(callbacks);

        remote.push(&[&refspec], Some(&mut push_options))?;

        // Push tags via subprocess (git2 doesn't support glob refspecs)
        let repo_path = repo.repository().path().parent().unwrap().to_path_buf();
        let mut tag_args = vec!["push", &mirror.name, "--tags"];
        if force { tag_args.push("--force"); }
        let _ = std::process::Command::new("git")
            .args(&tag_args)
            .current_dir(&repo_path)
            .output();

        Ok(())
    }

    /// Remove a mirror by platform and account
    pub fn remove_mirror_by_account(&self, platform: &str, account: &str) -> Result<()> {
        let mut config = self.load_config()?;
        
        let mirror = config.mirrors.iter()
            .find(|m| m.platform == platform && m.account_name == account)
            .ok_or_else(|| ToriiError::Mirror("Mirror not found".to_string()))?;
        
        let remote_name = mirror.name.clone();
        
        config.mirrors.retain(|m| !(m.platform == platform && m.account_name == account));
        self.save_config(&config)?;

        let repo = GitRepo::open(&self.repo_path)?;
        repo.repository().remote_delete(&remote_name)?;

        Ok(())
    }
    
    /// Remove a mirror by name (legacy)
    #[allow(dead_code)]
    pub fn remove_mirror(&self, name: &str) -> Result<()> {
        let mut config = self.load_config()?;
        
        config.mirrors.retain(|m| m.name != name);
        self.save_config(&config)?;

        let repo = GitRepo::open(&self.repo_path)?;
        repo.repository().remote_delete(name)?;

        Ok(())
    }

    /// Configure autofetch settings
    pub fn configure_autofetch(&self, enable: bool, interval: Option<u32>) -> Result<()> {
        let mut config = self.load_config()?;
        
        config.autofetch_enabled = enable;
        if let Some(interval_minutes) = interval {
            config.autofetch_interval_minutes = interval_minutes;
        }
        
        self.save_config(&config)?;
        
        if enable {
            let duration_str = format_duration(config.autofetch_interval_minutes);
            println!("✅ Autofetch enabled: every {}", duration_str);
            println!("💡 Torii will automatically fetch updates from all mirrors");
        } else {
            println!("❌ Autofetch disabled");
        }
        
        Ok(())
    }

    /// Show autofetch status
    pub fn show_autofetch_status(&self) -> Result<()> {
        let config = self.load_config()?;
        
        println!("🔄 Autofetch Configuration:");
        println!();
        
        if config.autofetch_enabled {
            let duration_str = format_duration(config.autofetch_interval_minutes);
            println!("  Status: ✅ Enabled");
            println!("  Interval: {}", duration_str);
            println!();
            println!("💡 Torii will automatically fetch from all mirrors every {}", duration_str);
        } else {
            println!("  Status: ❌ Disabled");
            println!();
            println!("💡 Enable with:");
            println!("   torii mirror autofetch --enable --interval <duration>");
            println!("   Examples: 10m, 30s, 2h, 1d");
        }
        
        Ok(())
    }
}
