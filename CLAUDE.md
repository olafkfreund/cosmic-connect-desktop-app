# CLAUDE.md - COSMIC Connect Development Guidelines

## Pre-Commit Workflow

### Code Simplification (REQUIRED)
Before creating any git commit, **ALWAYS** run the code-simplifier agent on changes:

```
Run code-simplifier agent on the changes we made
```

This ensures:
- Code clarity and consistency
- Removal of redundant patterns
- Better Rust idioms
- Improved maintainability
- Alignment with codebase conventions

**Exception:** Skip only if changes are trivial (typo fixes, comments only).

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
- Add `Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>`

## Project Architecture

### Plugin Development
- Each plugin in `cosmic-connect-core/src/plugins/`
- Implement both `Plugin` and `PluginFactory` traits
- Add config flag in `cosmic-connect-daemon/src/config.rs`
- Register factory in `cosmic-connect-daemon/src/main.rs`
- Follow existing plugin patterns (ping, battery, etc.)

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

*This project implements COSMIC Connect - a device connectivity solution for COSMIC Desktop*
