# CLAUDE.md - COSMIC Ext Connect Development Guidelines

## MANDATORY Pre-Commit Checks

### REQUIRED: Two-Step Pre-Commit Process

Before creating **ANY** git commit, you **MUST** run both checks:

#### Step 1: COSMIC Code Review (REQUIRED)

```bash
@cosmic-code-reviewer /pre-commit-check
```

This verifies:

- No hard-coded colors, dimensions, or radii
- No `.unwrap()` or `.expect()` calls
- Proper error handling and logging
- Theme integration correctness
- COSMIC Desktop best practices
- Architecture patterns

#### Step 2: Code Simplification (REQUIRED)

```bash
Run code-simplifier:code-simplifier agent on the changes we made
```

This ensures:

- Code clarity and consistency
- Removal of redundant patterns
- Better Rust idioms
- Improved maintainability
- Alignment with codebase conventions

**Exception:** Skip only if changes are trivial (typo fixes, comments only).

### Why Both Checks?

- **@cosmic-code-reviewer**: Catches COSMIC-specific issues (theming, widgets, patterns)
- **code-simplifier**: Optimizes Rust code quality and idioms
- Together they ensure high-quality, maintainable COSMIC Desktop code

## Naming Convention (Trademark Compliance)

This project follows the [COSMIC Trademark Policy](https://github.com/pop-os/cosmic-epoch/blob/master/TRADEMARK.md) for third-party projects:

| Category | Value |
|----------|-------|
| Binary prefix | `cosmic-ext-` |
| D-Bus namespace | `io.github.olafkfreund.CosmicExtConnect` |
| App IDs | `io.github.olafkfreund.CosmicExt*` |
| Nix pname | `cosmic-ext-connect` |
| Config dirs | `cosmic-ext-connect/` |
| Icon names | `cosmic-ext-connect-symbolic` |
| Crate prefix | `cosmic-ext-` |
| Rust identifier prefix | `cosmic_ext_` |

**Note:** The external `cosmic-connect-core` git dependency keeps its original name. In Cargo.toml, it is mapped via `package = "cosmic-connect-core"` with the workspace key `cosmic-ext-connect-core`.

## Development Standards

### Code Style

- Follow Rust idioms and conventions
- Use existing patterns from the codebase
- Prefer clarity over cleverness
- Keep functions focused and single-purpose

### Testing

- Write comprehensive unit tests for new plugins
- Test both success and error paths
- Use `create_test_device()` helper for consistency

### Documentation

- Document public APIs with doc comments
- Include protocol specifications in module docs
- Add usage examples for complex features
- Keep TODO comments with clear descriptions

### Commit Messages

- Use conventional commit format: `feat(scope): description`
- Include detailed body for complex changes
- Add `Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>`

## Project Architecture

### Crate Structure

| Crate | Description |
|-------|-------------|
| `cosmic-ext-connect-protocol` | Protocol library (plugins, discovery, pairing) |
| `cosmic-ext-applet-connect` | COSMIC panel applet |
| `cosmic-ext-connect-daemon` | Background daemon with D-Bus interface |
| `cosmic-ext-connect-manager` | Device management GUI |
| `cosmic-ext-messages` | Messaging app |
| `cosmic-ext-messages-popup` | Notification popup |
| `cosmic-ext-display-stream` | PipeWire capture + H.264 encoding |

### Plugin Development

- Each plugin in `cosmic-ext-connect-protocol/src/plugins/`
- Implement both `Plugin` and `PluginFactory` traits
- Add config flag in `cosmic-ext-connect-daemon/src/config.rs`
- Register factory in `cosmic-ext-connect-daemon/src/main.rs`
- Follow existing plugin patterns (ping, battery, etc.)

### D-Bus Interface

- **Service:** `io.github.olafkfreund.CosmicExtConnect`
- **Object path:** `/io/github/olafkfreund/CosmicExtConnect`
- **Interface:** `io.github.olafkfreund.CosmicExtConnect`
- **Open interface:** `io.github.olafkfreund.CosmicExtConnect.Open`

### Testing Strategy

- Unit tests in plugin modules
- Integration tests for daemon components
- Test with real devices when possible
- Document manual testing procedures

## Protocol References

- [Valent Protocol Documentation](https://valent.andyholmes.ca/documentation/protocol.html)
- [KDE Connect Community Wiki](https://community.kde.org/KDEConnect)
- [MPRIS2 Specification](https://specifications.freedesktop.org/mpris/latest/)

---

_This project implements COSMIC Ext Connect - a device connectivity solution for COSMIC Desktop_
