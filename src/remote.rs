use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::error::{Result, ToriiError};

/// Remote repository visibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Visibility {
    Public,
    Private,
    Internal, // GitLab only
}

/// Remote repository information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteRepo {
    pub name: String,
    pub description: Option<String>,
    pub visibility: Visibility,
    pub default_branch: String,
    pub url: String,
    pub ssh_url: String,
    pub clone_url: String,
}

/// Platform-specific API client trait
pub trait PlatformClient {
    /// Create a new repository
    fn create_repo(&self, name: &str, description: Option<&str>, visibility: Visibility) -> Result<RemoteRepo>;
    
    /// Delete a repository
    fn delete_repo(&self, owner: &str, repo: &str) -> Result<()>;
    
    /// Update repository settings
    fn update_repo(&self, owner: &str, repo: &str, settings: RepoSettings) -> Result<RemoteRepo>;
    
    /// Get repository information
    fn get_repo(&self, owner: &str, repo: &str) -> Result<RemoteRepo>;
    
    /// List user repositories
    fn list_repos(&self) -> Result<Vec<RemoteRepo>>;
    
    /// Set repository visibility
    fn set_visibility(&self, owner: &str, repo: &str, visibility: Visibility) -> Result<()>;
    
    /// Enable/disable features
    fn configure_features(&self, owner: &str, repo: &str, features: RepoFeatures) -> Result<()>;
}

/// Repository settings for updates
#[derive(Debug, Clone, Default)]
pub struct RepoSettings {
    pub description: Option<String>,
    pub homepage: Option<String>,
    pub visibility: Option<Visibility>,
    pub default_branch: Option<String>,
    pub has_issues: Option<bool>,
    pub has_wiki: Option<bool>,
    pub has_downloads: Option<bool>,
    pub allow_squash_merge: Option<bool>,
    pub allow_merge_commit: Option<bool>,
    pub allow_rebase_merge: Option<bool>,
}

/// Repository features configuration
#[derive(Debug, Clone, Default)]
pub struct RepoFeatures {
    pub issues: Option<bool>,
    pub wiki: Option<bool>,
    pub downloads: Option<bool>,
    pub projects: Option<bool>,
    pub discussions: Option<bool>,
}

/// GitHub API client (placeholder - requires reqwest)
pub struct GitHubClient {
    token: String,
    base_url: String,
}

impl GitHubClient {
    pub fn new(token: String) -> Self {
        Self {
            token,
            base_url: "https://api.github.com".to_string(),
        }
    }
    
    fn get_token() -> Result<String> {
        // Try to get token from environment or config
        if let Ok(token) = std::env::var("GITHUB_TOKEN") {
            return Ok(token);
        }
        
        if let Ok(token) = std::env::var("GH_TOKEN") {
            return Ok(token);
        }
        
        // Try to read from git config
        let output = std::process::Command::new("git")
            .args(&["config", "--global", "github.token"])
            .output()?;
        
        if output.status.success() {
            let token = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !token.is_empty() {
                return Ok(token);
            }
        }
        
        Err(ToriiError::InvalidConfig(
            "GitHub token not found. Set GITHUB_TOKEN env var or run: git config --global github.token YOUR_TOKEN".to_string()
        ))
    }
}

impl PlatformClient for GitHubClient {
    fn create_repo(&self, name: &str, description: Option<&str>, visibility: Visibility) -> Result<RemoteRepo> {
        // TODO: Implement with reqwest when added as dependency
        // For now, use GitHub CLI if available
        let mut args = vec!["repo", "create", name];
        
        match visibility {
            Visibility::Public => args.push("--public"),
            Visibility::Private => args.push("--private"),
            Visibility::Internal => args.push("--private"), // GitHub doesn't have Internal, default to private
        }
        
        if let Some(desc) = description {
            args.push("--description");
            args.push(desc);
        }
        
        let output = std::process::Command::new("gh")
            .args(&args)
            .output();
        
        match output {
            Ok(out) if out.status.success() => {
                println!("✅ Repository created on GitHub");
                // Return placeholder repo info
                Ok(RemoteRepo {
                    name: name.to_string(),
                    description: description.map(|s| s.to_string()),
                    visibility,
                    default_branch: "main".to_string(),
                    url: format!("https://github.com/USER/{}", name),
                    ssh_url: format!("git@github.com:USER/{}.git", name),
                    clone_url: format!("https://github.com/USER/{}.git", name),
                })
            }
            _ => {
                Err(ToriiError::InvalidConfig(
                    "Failed to create repository. Install GitHub CLI (gh) or set up API token".to_string()
                ))
            }
        }
    }
    
    fn delete_repo(&self, owner: &str, repo: &str) -> Result<()> {
        let output = std::process::Command::new("gh")
            .args(&["repo", "delete", &format!("{}/{}", owner, repo), "--yes"])
            .output();
        
        match output {
            Ok(out) if out.status.success() => {
                println!("✅ Repository deleted from GitHub");
                Ok(())
            }
            _ => {
                Err(ToriiError::InvalidConfig(
                    "Failed to delete repository. Install GitHub CLI (gh)".to_string()
                ))
            }
        }
    }
    
    fn update_repo(&self, owner: &str, repo: &str, settings: RepoSettings) -> Result<RemoteRepo> {
        let repo_name = format!("{}/{}", owner, repo);
        let mut args = vec!["repo", "edit", &repo_name];
        
        let mut temp_args = Vec::new();
        
        if let Some(desc) = &settings.description {
            temp_args.push("--description".to_string());
            temp_args.push(desc.clone());
        }
        
        if let Some(homepage) = &settings.homepage {
            temp_args.push("--homepage".to_string());
            temp_args.push(homepage.clone());
        }
        
        if let Some(vis) = &settings.visibility {
            match vis {
                Visibility::Public => temp_args.push("--visibility=public".to_string()),
                Visibility::Private => temp_args.push("--visibility=private".to_string()),
                Visibility::Internal => temp_args.push("--visibility=private".to_string()),
            }
        }
        
        if let Some(branch) = &settings.default_branch {
            temp_args.push("--default-branch".to_string());
            temp_args.push(branch.clone());
        }
        
        // Convert temp_args to string slices
        let arg_refs: Vec<&str> = temp_args.iter().map(|s| s.as_str()).collect();
        args.extend(arg_refs);
        
        let output = std::process::Command::new("gh")
            .args(&args)
            .output();
        
        match output {
            Ok(out) if out.status.success() => {
                println!("✅ Repository settings updated");
                self.get_repo(owner, repo)
            }
            _ => {
                Err(ToriiError::InvalidConfig(
                    "Failed to update repository settings".to_string()
                ))
            }
        }
    }
    
    fn get_repo(&self, owner: &str, repo: &str) -> Result<RemoteRepo> {
        let repo_name = format!("{}/{}", owner, repo);
        let output = std::process::Command::new("gh")
            .args(&["repo", "view", &repo_name, "--json", "name,description,visibility,defaultBranchRef,url,sshUrl"])
            .output();
        
        match output {
            Ok(out) if out.status.success() => {
                // Parse JSON output (simplified)
                Ok(RemoteRepo {
                    name: repo.to_string(),
                    description: None,
                    visibility: Visibility::Private,
                    default_branch: "main".to_string(),
                    url: format!("https://github.com/{}/{}", owner, repo),
                    ssh_url: format!("git@github.com:{}/{}.git", owner, repo),
                    clone_url: format!("https://github.com/{}/{}.git", owner, repo),
                })
            }
            _ => {
                Err(ToriiError::InvalidConfig(
                    "Failed to get repository information".to_string()
                ))
            }
        }
    }
    
    fn list_repos(&self) -> Result<Vec<RemoteRepo>> {
        let output = std::process::Command::new("gh")
            .args(&["repo", "list", "--json", "name,description,visibility", "--limit", "100"])
            .output();
        
        match output {
            Ok(out) if out.status.success() => {
                // Return empty list for now (would parse JSON in full implementation)
                Ok(Vec::new())
            }
            _ => {
                Err(ToriiError::InvalidConfig(
                    "Failed to list repositories".to_string()
                ))
            }
        }
    }
    
    fn set_visibility(&self, owner: &str, repo: &str, visibility: Visibility) -> Result<()> {
        let mut settings = RepoSettings::default();
        settings.visibility = Some(visibility);
        self.update_repo(owner, repo, settings)?;
        Ok(())
    }
    
    fn configure_features(&self, owner: &str, repo: &str, features: RepoFeatures) -> Result<()> {
        let repo_name = format!("{}/{}", owner, repo);
        let mut args = vec!["repo", "edit", &repo_name];
        
        let mut temp_args = Vec::new();
        
        if let Some(issues) = features.issues {
            temp_args.push(if issues { "--enable-issues".to_string() } else { "--disable-issues".to_string() });
        }
        
        if let Some(wiki) = features.wiki {
            temp_args.push(if wiki { "--enable-wiki".to_string() } else { "--disable-wiki".to_string() });
        }
        
        if let Some(projects) = features.projects {
            temp_args.push(if projects { "--enable-projects".to_string() } else { "--disable-projects".to_string() });
        }
        
        let arg_refs: Vec<&str> = temp_args.iter().map(|s| s.as_str()).collect();
        args.extend(arg_refs);
        
        let output = std::process::Command::new("gh")
            .args(&args)
            .output();
        
        match output {
            Ok(out) if out.status.success() => {
                println!("✅ Repository features configured");
                Ok(())
            }
            _ => {
                Err(ToriiError::InvalidConfig(
                    "Failed to configure repository features".to_string()
                ))
            }
        }
    }
}

/// GitLab API client (placeholder)
pub struct GitLabClient {
    token: Option<String>,
    base_url: String,
}

impl GitLabClient {
    pub fn new(token: Option<String>, base_url: Option<String>) -> Self {
        Self { 
            token,
            base_url: base_url.unwrap_or_else(|| "https://gitlab.com/api/v4".to_string()),
        }
    }
    
    pub fn with_url(token: String, base_url: String) -> Self {
        Self {
            token: Some(token),
            base_url,
        }
    }
}

impl PlatformClient for GitLabClient {
    fn create_repo(&self, name: &str, description: Option<&str>, visibility: Visibility) -> Result<RemoteRepo> {
        let token = self.token.as_ref()
            .ok_or_else(|| ToriiError::InvalidConfig(
                "GitLab token not found. Set GITLAB_TOKEN environment variable".to_string()
            ))?;

        let visibility_str = match visibility {
            Visibility::Public => "public",
            Visibility::Private => "private",
            Visibility::Internal => "internal",
        };

        let mut body = serde_json::json!({
            "name": name,
            "visibility": visibility_str,
        });

        if let Some(desc) = description {
            body["description"] = serde_json::json!(desc);
        }

        let client = reqwest::blocking::Client::new();
        let response = client
            .post(format!("{}/projects", self.base_url))
            .header("PRIVATE-TOKEN", token)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .map_err(|e| ToriiError::InvalidConfig(format!("GitLab API request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().unwrap_or_else(|_| "Unknown error".to_string());
            return Err(ToriiError::InvalidConfig(
                format!("GitLab API error: {}", error_text)
            ));
        }

        let project: serde_json::Value = response.json()
            .map_err(|e| ToriiError::InvalidConfig(format!("Failed to parse GitLab response: {}", e)))?;

        Ok(RemoteRepo {
            name: project["name"].as_str().unwrap_or(name).to_string(),
            description: project["description"].as_str().map(|s| s.to_string()),
            visibility,
            default_branch: project["default_branch"].as_str().unwrap_or("main").to_string(),
            url: project["web_url"].as_str().unwrap_or("").to_string(),
            ssh_url: project["ssh_url_to_repo"].as_str().unwrap_or("").to_string(),
            clone_url: project["http_url_to_repo"].as_str().unwrap_or("").to_string(),
        })
    }
    
    fn delete_repo(&self, owner: &str, repo: &str) -> Result<()> {
        let token = self.token.as_ref()
            .ok_or_else(|| ToriiError::InvalidConfig(
                "GitLab token not found. Set GITLAB_TOKEN environment variable".to_string()
            ))?;

        let path_str = format!("{}/{}", owner, repo);
        let project_path = urlencoding::encode(&path_str);
        let client = reqwest::blocking::Client::new();
        let response = client
            .delete(format!("{}/projects/{}", self.base_url, project_path))
            .header("PRIVATE-TOKEN", token)
            .send()
            .map_err(|e| ToriiError::InvalidConfig(format!("GitLab API request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().unwrap_or_else(|_| "Unknown error".to_string());
            return Err(ToriiError::InvalidConfig(
                format!("GitLab API error: {}", error_text)
            ));
        }

        Ok(())
    }
    
    fn update_repo(&self, owner: &str, repo: &str, settings: RepoSettings) -> Result<RemoteRepo> {
        let token = self.token.as_ref()
            .ok_or_else(|| ToriiError::InvalidConfig(
                "GitLab token not found. Set GITLAB_TOKEN environment variable".to_string()
            ))?;

        let path_str = format!("{}/{}", owner, repo);
        let project_path = urlencoding::encode(&path_str);
        let mut body = serde_json::json!({});

        if let Some(desc) = settings.description {
            body["description"] = serde_json::json!(desc);
        }
        if let Some(vis) = settings.visibility {
            let vis_str = match vis {
                Visibility::Public => "public",
                Visibility::Private => "private",
                Visibility::Internal => "internal",
            };
            body["visibility"] = serde_json::json!(vis_str);
        }
        if let Some(branch) = settings.default_branch {
            body["default_branch"] = serde_json::json!(branch);
        }

        let client = reqwest::blocking::Client::new();
        let response = client
            .put(format!("{}/projects/{}", self.base_url, project_path))
            .header("PRIVATE-TOKEN", token)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .map_err(|e| ToriiError::InvalidConfig(format!("GitLab API request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().unwrap_or_else(|_| "Unknown error".to_string());
            return Err(ToriiError::InvalidConfig(
                format!("GitLab API error: {}", error_text)
            ));
        }

        let project: serde_json::Value = response.json()
            .map_err(|e| ToriiError::InvalidConfig(format!("Failed to parse GitLab response: {}", e)))?;

        let visibility = match project["visibility"].as_str() {
            Some("public") => Visibility::Public,
            Some("internal") => Visibility::Internal,
            _ => Visibility::Private,
        };

        Ok(RemoteRepo {
            name: project["name"].as_str().unwrap_or(repo).to_string(),
            description: project["description"].as_str().map(|s| s.to_string()),
            visibility,
            default_branch: project["default_branch"].as_str().unwrap_or("main").to_string(),
            url: project["web_url"].as_str().unwrap_or("").to_string(),
            ssh_url: project["ssh_url_to_repo"].as_str().unwrap_or("").to_string(),
            clone_url: project["http_url_to_repo"].as_str().unwrap_or("").to_string(),
        })
    }
    
    fn get_repo(&self, owner: &str, repo: &str) -> Result<RemoteRepo> {
        let token = self.token.as_ref()
            .ok_or_else(|| ToriiError::InvalidConfig(
                "GitLab token not found. Set GITLAB_TOKEN environment variable".to_string()
            ))?;

        let path_str = format!("{}/{}", owner, repo);
        let project_path = urlencoding::encode(&path_str);
        let client = reqwest::blocking::Client::new();
        let response = client
            .get(format!("{}/projects/{}", self.base_url, project_path))
            .header("PRIVATE-TOKEN", token)
            .send()
            .map_err(|e| ToriiError::InvalidConfig(format!("GitLab API request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().unwrap_or_else(|_| "Unknown error".to_string());
            return Err(ToriiError::InvalidConfig(
                format!("GitLab API error: {}", error_text)
            ));
        }

        let project: serde_json::Value = response.json()
            .map_err(|e| ToriiError::InvalidConfig(format!("Failed to parse GitLab response: {}", e)))?;

        let visibility = match project["visibility"].as_str() {
            Some("public") => Visibility::Public,
            Some("internal") => Visibility::Internal,
            _ => Visibility::Private,
        };

        Ok(RemoteRepo {
            name: project["name"].as_str().unwrap_or(repo).to_string(),
            description: project["description"].as_str().map(|s| s.to_string()),
            visibility,
            default_branch: project["default_branch"].as_str().unwrap_or("main").to_string(),
            url: project["web_url"].as_str().unwrap_or("").to_string(),
            ssh_url: project["ssh_url_to_repo"].as_str().unwrap_or("").to_string(),
            clone_url: project["http_url_to_repo"].as_str().unwrap_or("").to_string(),
        })
    }
    
    fn list_repos(&self) -> Result<Vec<RemoteRepo>> {
        let token = self.token.as_ref()
            .ok_or_else(|| ToriiError::InvalidConfig(
                "GitLab token not found. Set GITLAB_TOKEN environment variable".to_string()
            ))?;

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(format!("{}/projects?membership=true&per_page=100", self.base_url))
            .header("PRIVATE-TOKEN", token)
            .send()
            .map_err(|e| ToriiError::InvalidConfig(format!("GitLab API request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().unwrap_or_else(|_| "Unknown error".to_string());
            return Err(ToriiError::InvalidConfig(
                format!("GitLab API error: {}", error_text)
            ));
        }

        let projects: Vec<serde_json::Value> = response.json()
            .map_err(|e| ToriiError::InvalidConfig(format!("Failed to parse GitLab response: {}", e)))?;

        Ok(projects.iter().map(|project| {
            let visibility = match project["visibility"].as_str() {
                Some("public") => Visibility::Public,
                Some("internal") => Visibility::Internal,
                _ => Visibility::Private,
            };

            RemoteRepo {
                name: project["name"].as_str().unwrap_or("").to_string(),
                description: project["description"].as_str().map(|s| s.to_string()),
                visibility,
                default_branch: project["default_branch"].as_str().unwrap_or("main").to_string(),
                url: project["web_url"].as_str().unwrap_or("").to_string(),
                ssh_url: project["ssh_url_to_repo"].as_str().unwrap_or("").to_string(),
                clone_url: project["http_url_to_repo"].as_str().unwrap_or("").to_string(),
            }
        }).collect())
    }
    
    fn set_visibility(&self, owner: &str, repo: &str, visibility: Visibility) -> Result<()> {
        let token = self.token.as_ref()
            .ok_or_else(|| ToriiError::InvalidConfig(
                "GitLab token not found. Set GITLAB_TOKEN environment variable".to_string()
            ))?;

        let path_str = format!("{}/{}", owner, repo);
        let project_path = urlencoding::encode(&path_str);
        let visibility_str = match visibility {
            Visibility::Public => "public",
            Visibility::Private => "private",
            Visibility::Internal => "internal",
        };

        let body = serde_json::json!({
            "visibility": visibility_str,
        });

        let client = reqwest::blocking::Client::new();
        let response = client
            .put(format!("{}/projects/{}", self.base_url, project_path))
            .header("PRIVATE-TOKEN", token)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .map_err(|e| ToriiError::InvalidConfig(format!("GitLab API request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().unwrap_or_else(|_| "Unknown error".to_string());
            return Err(ToriiError::InvalidConfig(
                format!("GitLab API error: {}", error_text)
            ));
        }

        Ok(())
    }
    
    fn configure_features(&self, owner: &str, repo: &str, features: RepoFeatures) -> Result<()> {
        let token = self.token.as_ref()
            .ok_or_else(|| ToriiError::InvalidConfig(
                "GitLab token not found. Set GITLAB_TOKEN environment variable".to_string()
            ))?;

        let path_str = format!("{}/{}", owner, repo);
        let project_path = urlencoding::encode(&path_str);
        let mut body = serde_json::json!({});

        if let Some(issues) = features.issues {
            body["issues_enabled"] = serde_json::json!(issues);
        }
        if let Some(wiki) = features.wiki {
            body["wiki_enabled"] = serde_json::json!(wiki);
        }

        let client = reqwest::blocking::Client::new();
        let response = client
            .put(format!("{}/projects/{}", self.base_url, project_path))
            .header("PRIVATE-TOKEN", token)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .map_err(|e| ToriiError::InvalidConfig(format!("GitLab API request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().unwrap_or_else(|_| "Unknown error".to_string());
            return Err(ToriiError::InvalidConfig(
                format!("GitLab API error: {}", error_text)
            ));
        }

        Ok(())
    }
}

/// Get appropriate platform client based on platform name
pub fn get_platform_client(platform: &str) -> Result<Box<dyn PlatformClient>> {
    match platform.to_lowercase().as_str() {
        "github" => {
            let token = GitHubClient::get_token()?;
            Ok(Box::new(GitHubClient::new(token)))
        }
        "gitlab" => {
            let token = std::env::var("GITLAB_TOKEN").ok();
            let base_url = std::env::var("GITLAB_URL").ok();
            Ok(Box::new(GitLabClient::new(token, base_url)))
        }
        "gitea" => {
            let token = std::env::var("GITEA_TOKEN").ok();
            let base_url = std::env::var("GITEA_URL")
                .unwrap_or_else(|_| "https://gitea.com".to_string());
            Ok(Box::new(GiteaClient::new(token, base_url)))
        }
        "forgejo" => {
            let token = std::env::var("FORGEJO_TOKEN").ok();
            let base_url = std::env::var("FORGEJO_URL")
                .unwrap_or_else(|_| "https://codeberg.org".to_string());
            Ok(Box::new(ForgejoClient::new(token, base_url)))
        }
        "codeberg" => {
            let token = std::env::var("CODEBERG_TOKEN").ok();
            Ok(Box::new(CodebergClient::new(token)))
        }
        _ => Err(ToriiError::InvalidConfig(
            format!("Unsupported platform: {}. Supported: github, gitlab, gitea, forgejo, codeberg", platform)
        )),
    }
}

// ============================================================================
// Gitea/Forgejo/Codeberg Clients
// ============================================================================

pub struct GiteaClient {
    token: Option<String>,
    base_url: String,
}

impl GiteaClient {
    pub fn new(token: Option<String>, base_url: String) -> Self {
        Self { token, base_url }
    }
}

pub struct ForgejoClient {
    token: Option<String>,
    base_url: String,
}

impl ForgejoClient {
    pub fn new(token: Option<String>, base_url: String) -> Self {
        Self { token, base_url }
    }
}

pub struct CodebergClient {
    token: Option<String>,
}

impl CodebergClient {
    pub fn new(token: Option<String>) -> Self {
        Self { token }
    }
}

// Placeholder implementations - will be completed with API calls
impl PlatformClient for GiteaClient {
    fn create_repo(&self, _name: &str, _description: Option<&str>, _visibility: Visibility) -> Result<RemoteRepo> {
        Err(ToriiError::InvalidConfig("Gitea API not yet implemented".to_string()))
    }
    fn delete_repo(&self, _owner: &str, _repo: &str) -> Result<()> {
        Err(ToriiError::InvalidConfig("Gitea API not yet implemented".to_string()))
    }
    fn update_repo(&self, _owner: &str, _repo: &str, _settings: RepoSettings) -> Result<RemoteRepo> {
        Err(ToriiError::InvalidConfig("Gitea API not yet implemented".to_string()))
    }
    fn get_repo(&self, _owner: &str, _repo: &str) -> Result<RemoteRepo> {
        Err(ToriiError::InvalidConfig("Gitea API not yet implemented".to_string()))
    }
    fn list_repos(&self) -> Result<Vec<RemoteRepo>> {
        Err(ToriiError::InvalidConfig("Gitea API not yet implemented".to_string()))
    }
    fn set_visibility(&self, _owner: &str, _repo: &str, _visibility: Visibility) -> Result<()> {
        Err(ToriiError::InvalidConfig("Gitea API not yet implemented".to_string()))
    }
    fn configure_features(&self, _owner: &str, _repo: &str, _features: RepoFeatures) -> Result<()> {
        Err(ToriiError::InvalidConfig("Gitea API not yet implemented".to_string()))
    }
}

impl PlatformClient for ForgejoClient {
    fn create_repo(&self, _name: &str, _description: Option<&str>, _visibility: Visibility) -> Result<RemoteRepo> {
        Err(ToriiError::InvalidConfig("Forgejo API not yet implemented".to_string()))
    }
    fn delete_repo(&self, _owner: &str, _repo: &str) -> Result<()> {
        Err(ToriiError::InvalidConfig("Forgejo API not yet implemented".to_string()))
    }
    fn update_repo(&self, _owner: &str, _repo: &str, _settings: RepoSettings) -> Result<RemoteRepo> {
        Err(ToriiError::InvalidConfig("Forgejo API not yet implemented".to_string()))
    }
    fn get_repo(&self, _owner: &str, _repo: &str) -> Result<RemoteRepo> {
        Err(ToriiError::InvalidConfig("Forgejo API not yet implemented".to_string()))
    }
    fn list_repos(&self) -> Result<Vec<RemoteRepo>> {
        Err(ToriiError::InvalidConfig("Forgejo API not yet implemented".to_string()))
    }
    fn set_visibility(&self, _owner: &str, _repo: &str, _visibility: Visibility) -> Result<()> {
        Err(ToriiError::InvalidConfig("Forgejo API not yet implemented".to_string()))
    }
    fn configure_features(&self, _owner: &str, _repo: &str, _features: RepoFeatures) -> Result<()> {
        Err(ToriiError::InvalidConfig("Forgejo API not yet implemented".to_string()))
    }
}

impl PlatformClient for CodebergClient {
    fn create_repo(&self, _name: &str, _description: Option<&str>, _visibility: Visibility) -> Result<RemoteRepo> {
        Err(ToriiError::InvalidConfig("Codeberg API not yet implemented".to_string()))
    }
    fn delete_repo(&self, _owner: &str, _repo: &str) -> Result<()> {
        Err(ToriiError::InvalidConfig("Codeberg API not yet implemented".to_string()))
    }
    fn update_repo(&self, _owner: &str, _repo: &str, _settings: RepoSettings) -> Result<RemoteRepo> {
        Err(ToriiError::InvalidConfig("Codeberg API not yet implemented".to_string()))
    }
    fn get_repo(&self, _owner: &str, _repo: &str) -> Result<RemoteRepo> {
        Err(ToriiError::InvalidConfig("Codeberg API not yet implemented".to_string()))
    }
    fn list_repos(&self) -> Result<Vec<RemoteRepo>> {
        Err(ToriiError::InvalidConfig("Codeberg API not yet implemented".to_string()))
    }
    fn set_visibility(&self, _owner: &str, _repo: &str, _visibility: Visibility) -> Result<()> {
        Err(ToriiError::InvalidConfig("Codeberg API not yet implemented".to_string()))
    }
    fn configure_features(&self, _owner: &str, _repo: &str, _features: RepoFeatures) -> Result<()> {
        Err(ToriiError::InvalidConfig("Codeberg API not yet implemented".to_string()))
    }
}
