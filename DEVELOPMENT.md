# Torii Development Guide

## Project Structure

```
torii/
├── src/
│   ├── main.rs          # Entry point
│   ├── cli.rs           # CLI argument parsing and command execution
│   ├── core.rs          # Core git operations wrapper
│   ├── error.rs         # Error types
│   ├── snapshot.rs      # Local snapshot system
│   └── mirror.rs        # Multi-platform mirroring
├── Cargo.toml           # Dependencies and metadata
└── docs/                # Documentation (i18n)
```

## Architecture

Torii follows a modular architecture designed to support multiple frontends:

```
┌─────────────────────────────────────┐
│  Presentation Layer                 │
│  ┌──────────┐  ┌──────────┐  ┌───┐ │
│  │   CLI    │  │   TUI    │  │GUI│ │
│  │ (current)│  │ (future) │  │(f)│ │
│  └──────────┘  └──────────┘  └───┘ │
└─────────────────────────────────────┘
           ↓          ↓          ↓
┌─────────────────────────────────────┐
│  Core Business Logic (Rust)         │
│  • Git operations (core.rs)         │
│  • Snapshot system (snapshot.rs)    │
│  • Mirror management (mirror.rs)    │
│  • Error handling (error.rs)        │
└─────────────────────────────────────┘
           ↓
┌─────────────────────────────────────┐
│  External Dependencies              │
│  • git2 (libgit2 bindings)          │
│  • serde (serialization)            │
│  • chrono (timestamps)              │
└─────────────────────────────────────┘
```

## Building

### Development build
```bash
cargo build
```

### Release build (optimized)
```bash
cargo build --release
```

### Run tests
```bash
cargo test
```

## Current Features (v0.1.0)

### ✅ Implemented
- **Simplified Git Commands**
  - `torii init` - Initialize repository
  - `torii save` - Simplified commit
  - `torii sync` - Pull + Push
  - `torii status` - Repository status

- **Snapshot System**
  - `torii snapshot create` - Create local snapshot
  - `torii snapshot list` - List all snapshots
  - `torii snapshot restore <id>` - Restore snapshot
  - `torii snapshot delete <id>` - Delete snapshot
  - `torii snapshot config` - Configure auto-snapshots

- **Mirror Management**
  - `torii mirror add <platform> <url>` - Add mirror
  - `torii mirror list` - List mirrors
  - `torii mirror sync` - Sync to all mirrors
  - `torii mirror remove <name>` - Remove mirror

### 🚧 Planned
- **TUI (Terminal User Interface)** using Ratatui
- **GUI (Graphical User Interface)** using Tauri
- **Auto-snapshot triggers** (session events, time-based)
- **Platform API integration** (GitHub, GitLab, Bitbucket)
- **PR/Issue management** from CLI/TUI/GUI
- **Conflict resolution UI**
- **Interactive rebase**
- **Git hooks integration**

## Development Workflow

1. **Feature branches**: Create feature branches for new work
2. **Testing**: Add tests for new functionality
3. **Documentation**: Update docs for user-facing changes
4. **Commit messages**: Use conventional commits format

## Adding New Commands

1. Add command variant to `Commands` enum in `cli.rs`
2. Implement handler in `Cli::execute()` method
3. Add core functionality in appropriate module
4. Add tests
5. Update documentation

## Code Style

- Follow Rust standard formatting (`cargo fmt`)
- Run clippy before committing (`cargo clippy`)
- Keep functions focused and small
- Document public APIs
- Use descriptive error messages

## Dependencies

- **clap**: CLI argument parsing
- **git2**: Git operations (libgit2 bindings)
- **anyhow**: Error handling
- **thiserror**: Custom error types
- **serde/serde_json**: Configuration serialization
- **chrono**: Timestamp handling

## Future Roadmap

### Phase 1: CLI (Current)
- ✅ Basic git operations
- ✅ Snapshot system
- ✅ Mirror management
- 🚧 Auto-snapshot daemon
- 🚧 Platform API integration

### Phase 2: TUI
- Terminal UI with Ratatui
- Interactive file staging
- Visual diff viewer
- Branch visualization
- Snapshot browser

### Phase 3: GUI
- Desktop app with Tauri
- All TUI features
- Drag-and-drop operations
- Visual merge conflict resolution
- Settings panel

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for contribution guidelines.

## License

MIT License - see [LICENSE](LICENSE) for details.
