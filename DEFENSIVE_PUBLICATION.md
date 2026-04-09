# Defensive Publication - Torii Innovations

**Read this in other languages**: [Español](docs/i18n/defensive/DEFENSIVE_PUBLICATION.es.md) | [日本語](docs/i18n/defensive/DEFENSIVE_PUBLICATION.ja.md) | [Deutsch](docs/i18n/defensive/DEFENSIVE_PUBLICATION.de.md) | [Français](docs/i18n/defensive/DEFENSIVE_PUBLICATION.fr.md) | [Italiano](docs/i18n/defensive/DEFENSIVE_PUBLICATION.it.md) | [Ελληνικά](docs/i18n/defensive/DEFENSIVE_PUBLICATION.el.md) | [Русский](docs/i18n/defensive/DEFENSIVE_PUBLICATION.ru.md) | [한국어](docs/i18n/defensive/DEFENSIVE_PUBLICATION.ko.md) | [中文](docs/i18n/defensive/DEFENSIVE_PUBLICATION.zh.md) | [العربية](docs/i18n/defensive/DEFENSIVE_PUBLICATION.ar.md)

**Publication Date**: April 9, 2026  
**Author**: Torii Project  
**Repository**: https://gitlab.com/paskidev/torii  
**Purpose**: Establish prior art to prevent patent claims on these innovations

---

## EXECUTIVE SUMMARY

This defensive publication documents the following **novel technical innovations**:

1. ✅ **Local Snapshot System with Session-Based Auto-Capture** - Unique integration of OS session events with git state preservation
2. ✅ **Centralized Cross-Platform PR/Issue Management** - First git client with bidirectional, interactive management across GitHub/GitLab/Bitbucket from single interface  
3. ✅ **Shared Core Architecture (GUI + TUI)** - Unique in git clients: perfect feature parity between Tauri GUI and Ratatui TUI

**Innovations NOT claimed**: 
- AI/ML algorithms or code analysis tools (use existing tools if/when needed)
- Basic git operations (standard git functionality)
- Static code analysis techniques (well-established prior art)

**Future innovations** will be added to this document with their own publication dates as Torii evolves.

---

## LEGAL NOTICE

This document constitutes a **defensive publication** under:
- EU Patent Convention Article 54(2) - Novelty
- US Patent Act 35 U.S.C. § 102 - Conditions for patentability
- WIPO Patent Cooperation Treaty Article 15

By publishing these technical innovations, we create **irrefutable prior art** that prevents 
any third party from obtaining valid patent protection for these concepts after this date.

**Evidence of Publication**:
- Git Commit Hash: [first commit hash]
- Publication Date: 2026-04-09
- Repository URL: https://gitlab.com/paskidev/torii
- Archive: https://archive.org (web archive snapshot)
- DOI: [optional - Zenodo/figshare DOI]

---

## INNOVATION 1: Local Snapshot System with Session-Based Auto-Capture

### Problem Solved
Traditional git workflows lose intermediate changes when:
- System crashes or unexpected shutdowns occur
- User forgets to commit before closing session
- Working on experimental changes across multiple sessions
- Need to preserve work-in-progress without polluting remote history

### Technical Innovation

**Auto-Snapshot Trigger Mechanism**:
- Hooks into OS session management events (logout, shutdown)
- Monitors application close events
- Configurable time-based triggers (every N minutes)
- Smart detection of "significant changes" to avoid noise snapshots

**Storage Architecture**:
```
.torii/snapshots/
  ├── 2026-04-09T14-30-00_session_close/
  │   ├── metadata.json (timestamp, trigger, diff stats)
  │   ├── full_state.bundle (git bundle of complete state)
  │   └── diff_summary.txt
  ├── 2026-04-09T15-45-00_auto_interval/
  └── ...
```

**Key Features**:
1. **Local-only**: Never pushes to remote automatically
2. **Full history preservation**: Captures all intermediate states
3. **Auto-cleanup**: Configurable retention (e.g., keep 30 days)
4. **Fast restore**: One-click rollback to any snapshot
5. **Diff visualization**: Compare current state with any snapshot
6. **Configurable frequency**: Per-project or global settings

**Differentiators from Existing Solutions**:
- `git stash`: Only one state, loses history between stashes
- `git worktree`: Requires manual management, no auto-capture
- `git reflog`: Limited to committed changes, not working directory
- IDE local history: Not git-aware, no cross-tool compatibility

### Algorithm Pseudocode

```rust
fn on_session_event(event: SessionEvent) {
    if should_create_snapshot(event) {
        let snapshot = Snapshot {
            timestamp: now(),
            trigger: event.type,
            working_dir: capture_working_directory(),
            staged: capture_staged_changes(),
            git_state: bundle_full_repository(),
        };
        
        save_snapshot_local(snapshot);
        cleanup_old_snapshots(retention_policy);
    }
}

fn should_create_snapshot(event: SessionEvent) -> bool {
    match event {
        SessionEvent::Shutdown | SessionEvent::Logout => true,
        SessionEvent::AppClose => has_unsaved_changes(),
        SessionEvent::TimeInterval => significant_changes_since_last(),
    }
}
```

**Prior Art Date**: 2026-04-09

---

## INNOVATION 2: Bidirectional Mirror Synchronization Architecture

### Problem Solved
Developers maintaining mirrors across multiple git platforms (GitHub, GitLab, Bitbucket) face:
- Manual synchronization overhead
- Pull requests/issues scattered across platforms
- No unified view of contributions
- Risk of mirrors becoming out of sync

### Technical Innovation

**Mirror Sync Engine**:
```
Main Repository (source of truth)
    ↕️ Bidirectional Sync
Mirror 1 (GitHub) ← PRs, Issues
Mirror 2 (GitLab) ← PRs, Issues
Mirror 3 (Bitbucket) ← PRs, Issues
    ↓
Centralized Management Interface (Torii)
```

**Architecture Components**:

1. **Sync Daemon**:
   - Monitors main repo for changes (webhooks + polling)
   - Propagates commits to all mirrors atomically
   - Handles force-push conflicts intelligently
   - Maintains sync state database

2. **PR/Issue Aggregator**:
   - Fetches PRs/issues from all mirror platforms
   - Normalizes data to unified format
   - Provides single interface for management
   - Syncs actions back to origin platform (comment, merge, close)

3. **Conflict Resolution**:
   - Detects divergent histories across mirrors
   - Presents unified diff view
   - Allows selection of authoritative source
   - Auto-merges when safe, prompts when manual intervention needed

**Data Structure**:
```rust
struct MirrorConfig {
    main_repo: Repository,
    mirrors: Vec<Mirror>,
    sync_strategy: SyncStrategy,
}

struct Mirror {
    platform: Platform, // GitHub, GitLab, etc.
    url: String,
    credentials: SecureCredentials,
    sync_enabled: bool,
    pull_requests: Vec<PullRequest>,
    issues: Vec<Issue>,
}

enum SyncStrategy {
    MainToMirrors,      // One-way: main → mirrors
    Bidirectional,      // Two-way sync with conflict detection
    MirrorPreferred,    // Mirrors can override main (advanced)
}
```

**Key Innovations**:
1. **Atomic multi-mirror push**: All mirrors updated or none
2. **Cross-platform PR management**: Respond to GitHub PR from Torii, syncs back
3. **Smart conflict detection**: Understands git semantics, not just dumb sync
4. **State reconciliation**: Can recover from partial failures
5. **Bandwidth optimization**: Only syncs deltas, not full repos

**Prior Art Date**: 2026-04-09

---

## INNOVATION 3: Centralized Cross-Platform PR/Issue Management

### Problem Solved
Developers maintaining mirrors across multiple git platforms face scattered workflows:
- PRs on GitHub require GitHub UI
- Issues on GitLab require GitLab UI  
- No unified dashboard for contributions across platforms
- Context switching between platforms wastes time
- Can't batch-respond to multiple platforms

### Technical Innovation

**Unified Management Interface**:
```
Pull Requests from:
  • GitHub mirror
  • GitLab mirror  
  • Bitbucket mirror
      ↓
  Torii Unified Dashboard
      ↓
Actions (comment/merge/close) → Sync back to origin platform
```

**Key Innovation - Bidirectional Action Sync**:
Unlike passive aggregators (RSS readers, dashboards), Torii allows:
- View GitHub PR in Torii
- Comment/review in Torii interface
- Action syncs back to GitHub via API
- Notifications to PR author come from GitHub (transparent)

**Architecture**:
```rust
struct UnifiedPRManager {
    aggregator: PRAggregator,
    action_dispatcher: ActionDispatcher,
}

impl UnifiedPRManager {
    // Fetch from all platforms
    async fn fetch_all_prs(&self) -> Vec<UnifiedPR> {
        let github_prs = self.aggregator.fetch_github().await;
        let gitlab_prs = self.aggregator.fetch_gitlab().await;
        let bitbucket_prs = self.aggregator.fetch_bitbucket().await;
        
        normalize_and_merge(github_prs, gitlab_prs, bitbucket_prs)
    }
    
    // Action dispatches back to origin
    async fn comment_on_pr(&self, pr: &UnifiedPR, comment: &str) {
        match pr.origin_platform {
            Platform::GitHub => {
                self.action_dispatcher
                    .github_comment(pr.origin_id, comment)
                    .await
            }
            Platform::GitLab => {
                self.action_dispatcher
                    .gitlab_comment(pr.origin_id, comment)
                    .await
            }
            // ...
        }
    }
}
```

**Normalized Data Model**:
```rust
struct UnifiedPR {
    id: String,                    // Torii internal ID
    origin_platform: Platform,     // Where it came from
    origin_id: String,             // Platform-specific ID
    title: String,
    author: String,
    state: PRState,               // Open/Merged/Closed
    comments: Vec<Comment>,
    reviews: Vec<Review>,
    files_changed: Vec<FileDiff>,
}
```

**Differentiators from Existing Tools**:
- GitHub/GitLab/Bitbucket native UIs: Single platform only
- GitHub CLI (`gh`): GitHub only, no unified view
- GitLab CLI (`glab`): GitLab only
- Multi-account tools (Shift, Station): Browser wrappers, no native integration
- RSS aggregators: Read-only, can't take actions
- Zapier/IFTTT: Automation, not interactive management

**This innovation is unique**: No existing git client provides interactive, 
bidirectional PR/issue management across multiple platforms from a single 
native interface.

**Prior Art Date**: 2026-04-09

---

## INNOVATION 4: Shared Core Architecture (GUI + TUI)

### Problem Solved
Git clients typically have separate implementations for GUI and TUI:
- GitKraken (GUI only), LazyGit (TUI only) - completely different codebases
- Feature parity issues between interfaces
- Duplicate bug fixes required
- Inconsistent behavior

### Technical Innovation

**Single Core, Multiple Frontends**:
```
┌─────────────────────────────────────┐
│  Presentation Layer                 │
│  ┌──────────┐      ┌──────────┐   │
│  │   Tauri  │      │ Ratatui  │   │
│  │   (GUI)  │      │  (TUI)   │   │
│  └──────────┘      └──────────┘   │
└─────────────────────────────────────┘
           ↓                ↓
┌─────────────────────────────────────┐
│  Shared Core (Rust)                 │
│  • All git operations               │
│  • All business logic               │
│  • State management                 │
│  • Plugin system                    │
└─────────────────────────────────────┘
```

**Implementation**:
```rust
// Core trait - single source of truth
pub trait GitClient {
    fn commit(&self, msg: &str) -> Result<Oid>;
    fn create_snapshot(&self) -> Result<Snapshot>;
    fn sync_mirrors(&self) -> Result<SyncReport>;
    // ... all operations
}

// Shared business logic
pub struct ToriiCore {
    repo: Repository,
    config: Config,
    // ... all state
}

impl GitClient for ToriiCore {
    // Implementation once, used by both GUI and TUI
    fn commit(&self, msg: &str) -> Result<Oid> {
        // Core logic here
    }
}

// GUI wrapper (Tauri)
pub struct TauriGitClient {
    core: Arc<ToriiCore>,
    // Only UI state here
}

impl TauriGitClient {
    fn commit_with_ui(&self, msg: &str) -> Result<Oid> {
        // Show loading spinner
        let result = self.core.commit(msg)?;  // Delegate to core
        // Update UI
        Ok(result)
    }
}

// TUI wrapper (Ratatui)
pub struct RatatuiGitClient {
    core: Arc<ToriiCore>,
    // Only terminal UI state here
}
```

**Key Benefits**:
1. Fix once, both UIs benefit
2. Features automatically available in both
3. Test core once, trust both UIs
4. Plugin system works in both interfaces

**Differentiators**:
- Vim/Neovim: TUI primary, GUI wrappers are separate projects
- Emacs: Similar issue, GUI/terminal have different behaviors
- VSCode: GUI only (terminal integration is different)
- Git CLI vs GUI clients: Completely separate implementations

**This is unique**: No git client currently shares core logic between 
native GUI (Tauri) and native TUI (Ratatui) with feature parity.

**Prior Art Date**: 2026-04-09

---

## FUTURE INNOVATIONS (Reserved)

### Problem Solved
Most git clients have separate implementations for GUI and CLI, leading to:
- Feature parity issues (GUI has features CLI doesn't)
- Duplicate code and bugs
- Inconsistent behavior between interfaces
- Higher maintenance burden

### Technical Innovation

**Layered Architecture**:
```
┌─────────────────────────────────────┐
│  Presentation Layer                 │
│  ┌──────────┐      ┌──────────┐   │
│  │   Tauri  │      │ Ratatui  │   │
│  │   (GUI)  │      │  (TUI)   │   │
│  └──────────┘      └──────────┘   │
└─────────────────────────────────────┘
           ↓                ↓
┌─────────────────────────────────────┐
│  Core Business Logic (Rust)         │
│  • GitOperations trait              │
│  • SnapshotManager                  │
│  • MirrorSync                       │
│  • AIAnalyzer                       │
│  • Config & State Management        │
└─────────────────────────────────────┘
           ↓
┌─────────────────────────────────────┐
│  git2-rs (libgit2 bindings)         │
└─────────────────────────────────────┘
```

**Core Abstraction**:
```rust
// Single source of truth for all git operations
pub trait GitOperations {
    fn commit(&self, message: &str) -> Result<Oid>;
    fn create_snapshot(&self) -> Result<Snapshot>;
    fn sync_mirrors(&self) -> Result<SyncReport>;
    fn analyze_pr(&self, pr_id: &str) -> Result<PRScore>;
}

// GUI implementation
impl GitOperations for TauriGitClient {
    fn commit(&self, message: &str) -> Result<Oid> {
        self.core.commit(message)  // Delegates to core
    }
    // ... UI-specific rendering logic
}

// TUI implementation  
impl GitOperations for RatatuiGitClient {
    fn commit(&self, message: &str) -> Result<Oid> {
        self.core.commit(message)  // Same core logic!
    }
    // ... Terminal-specific rendering
}
```

**Key Benefits**:
1. **Perfect feature parity**: Both interfaces get all features
2. **Single source of bugs**: Fix once, fixed everywhere
3. **Consistent behavior**: Identical git operations
4. **Easy testing**: Test core once, trust both UIs
5. **Plugin system**: Plugins work in both GUI and TUI

**Prior Art Date**: 2026-04-09

---

## FUTURE INNOVATIONS (Reserved)

This section reserves space for innovations to be added as Torii evolves.
Each future innovation will be documented with its own publication date
to establish prior art at the time of implementation.

### How to Add Future Innovations

When implementing a new feature that you believe is novel:

1. **Document thoroughly** in this file:
   - Problem solved
   - Technical approach  
   - Key differentiators from existing solutions
   - Code examples/pseudocode
   - Publication date

2. **Commit to repository** to establish timestamp

3. **Update archive** (Archive.org, IP.com if using)

4. **Consider defensive publication** to specialized databases:
   - IP.com Defensive Publications (free)
   - Research papers (arXiv, ResearchGate)
   - Technical blog posts with timestamps

### Placeholder Sections

**INNOVATION 5: [To Be Determined]**
- Reserved for future technical innovations
- Publication Date: TBD
- Example: AI-powered code analysis (if implemented with novel approach)

**INNOVATION 6: [To Be Determined]**  
- Reserved for future technical innovations
- Publication Date: TBD

**INNOVATION 7: [To Be Determined]**
- Reserved for future technical innovations
- Publication Date: TBD

**Note**: Empty placeholders do NOT establish prior art. Only documented,
published innovations with specific dates are legally defensible.

---

## PUBLICATION RECORD

This document was published on **April 9, 2026** in the following locations:

1. ✅ GitLab Repository: https://gitlab.com/paskidev/torii
2. ✅ Git Commit Hash: [to be filled on first commit]
3. ✅ Web Archive: https://archive.org/web/
4. ⏳ IP.com Defensive Publications (optional, submit after repo creation)
5. ⏳ Zenodo DOI (optional, for academic citation)

---

## LEGAL EFFECT

Any patent application filed AFTER April 9, 2026 claiming these innovations 
is subject to invalidation based on this prior art publication.

This publication is made available under CC0 1.0 Universal (Public Domain) 
for the PURPOSE OF ESTABLISHING PRIOR ART ONLY. The actual source code 
implementing these innovations is licensed separately under the Torii 
Source-Available License.

---

## CONTACT

For questions about this defensive publication:
- Email: paski@paski.dev
- Legal inquiries: paski@paski.dev

---

**Document Version**: 1.0  
**Last Updated**: 2026-04-09  
**SHA-256 Hash**: [will be generated on commit]

---

## APPENDIX: Adding New Innovations

### When to Document a New Feature

Document a feature in this file if it meets ALL criteria:

1. ✅ **Novel combination**: Even if individual components exist, your combination/integration is unique
2. ✅ **Non-obvious**: A skilled developer wouldn't immediately think of this approach
3. ✅ **Specific implementation**: Not just an idea, but a concrete technical approach
4. ❌ **NOT just using existing tools**: Don't claim "innovation" for using Rust/Tauri/git2-rs

### Template for New Innovations

```markdown
## INNOVATION X: [Descriptive Title]

### Problem Solved
[What pain point does this address?]

### Technical Innovation
[Your specific approach - be detailed]

**Architecture/Algorithm**:
[Code examples, pseudocode, diagrams]

**Differentiators from Existing Solutions**:
- Existing Tool A: [Why your approach is different]
- Existing Tool B: [Why your approach is different]

**Key Novel Aspects**:
1. [Specific unique element]
2. [Specific unique element]

**Prior Art Date**: [YYYY-MM-DD of commit]
```

### Publication Checklist

When adding a new innovation:

- [ ] Document thoroughly in this file
- [ ] Commit to git repository (establishes timestamp)
- [ ] Update "Last Updated" date in this document
- [ ] Take Archive.org snapshot of repository
- [ ] (Optional) Submit to IP.com defensive publications
- [ ] (Optional) Write technical blog post with date
- [ ] Update README.md Prior Art Notice with new date range

### Example: How to Handle Similar Features

**Scenario**: You implement "Smart Merge Conflict Resolution with ML"

**Bad approach** ❌:
```
"We invented AI-powered merge conflict resolution"
```
(This already exists: Git's merge strategies, Copilot suggestions, etc.)

**Good approach** ✅:
```
"Novel integration of ML-based conflict resolution with visual 
3-way diff in native git client, using repository-specific 
training on historical merge decisions"

Differentiators:
- GitHub Copilot: Suggests code, doesn't resolve conflicts
- Git mergetool: Manual only, no ML assistance  
- Our approach: Learns from YOUR past decisions, not generic training
```

Focus on what makes YOUR implementation unique, not the general concept.
