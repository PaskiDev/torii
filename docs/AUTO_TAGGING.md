# Release tagging

Torii uses [Conventional Commits](https://www.conventionalcommits.org/) to determine version bumps.

## Create a release

```bash
# Preview the next version without creating the tag
torii tag release --dry-run

# Create the tag (auto-detects bump type from commits since last tag)
torii tag release

# Force a specific bump type
torii tag release --bump major
torii tag release --bump minor
torii tag release --bump patch

# Push the tag to remote (included automatically in torii sync)
torii sync --push
```

## How version bumps are determined

Torii reads all commits since the last tag and picks the highest applicable bump:

| Commit prefix | Version bump |
|---------------|-------------|
| `feat!:` or `BREAKING CHANGE` | Major (x.0.0) |
| `feat:` | Minor (0.x.0) |
| `fix:`, `perf:` | Patch (0.0.x) |
| `docs:`, `chore:`, `refactor:`, etc. | No tag created |

If no releasable commits are found since the last tag, `torii tag release` will exit with an error. Use `--bump` to override.

## Manual tagging

For full control, use `torii tag create` directly:

```bash
torii tag create v2.0.0 -m "Major release — new VCS backend"
torii sync --push
```
