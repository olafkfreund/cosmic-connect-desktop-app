# Contributing to COSMIC Connect

Thank you for your interest in contributing to COSMIC Connect! This guide will help you get started.

## Table of Contents

- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Claude Code Skill](#claude-code-skill)
- [Code Style](#code-style)
- [Commit Guidelines](#commit-guidelines)
- [Pull Request Process](#pull-request-process)
- [Testing](#testing)
- [Documentation](#documentation)

## Getting Started

COSMIC Connect is a device connectivity solution for COSMIC Desktop, implementing the KDE Connect protocol. Before contributing, familiarize yourself with:

- [KDE Connect Protocol](https://community.kde.org/KDEConnect)
- [COSMIC Desktop](https://system76.com/cosmic)
- [libcosmic Book](https://pop-os.github.io/libcosmic-book/)

## Development Setup

### Prerequisites

#### NixOS (Recommended)
```bash
# The flake.nix includes all dependencies
nix develop
```

#### Ubuntu/Debian
```bash
sudo apt install cargo cmake just libexpat1-dev libfontconfig-dev \
    libfreetype-dev libxkbcommon-dev pkgconf libssl-dev
```

### Clone and Build

```bash
git clone https://github.com/olafkfreund/cosmic-connect-desktop-app
cd cosmic-connect-desktop-app
nix develop  # Or ensure dependencies are installed
just build
```

### Running Tests

```bash
just test           # Run all tests
just lint           # Run clippy linter
just fmt            # Format code
```

## Claude Code Skill

This project includes a custom Claude Code skill to assist with COSMIC Desktop development best practices.

### Installation

Install the skill for AI-assisted development:

```bash
./.claude/skills/install.sh
```

After installation, **restart Claude Code** to activate the skill.

### Using the Skill

The skill provides 7 specialized agents:

#### Quick Pre-Commit Check
```bash
@cosmic-code-reviewer /pre-commit-check
```

#### Architecture Review
```bash
@cosmic-architect review this application structure
@cosmic-architect /suggest-refactoring
```

#### Theming Audit
```bash
@cosmic-theme-expert /audit-theming
@cosmic-theme-expert check for hard-coded values
```

#### Applet Development
```bash
@cosmic-applet-specialist review this applet
@cosmic-applet-specialist /fix-popup
```

#### Error Handling
```bash
@cosmic-error-handler /remove-unwraps
@cosmic-error-handler audit error handling
```

#### Performance
```bash
@cosmic-performance-optimizer /find-bottlenecks
@cosmic-performance-optimizer check for blocking operations
```

#### Comprehensive Review
```bash
@cosmic-code-reviewer /full-review
```

See `.claude/skills/cosmic-ui-design-skill/README.md` for complete documentation.

## Code Style

### Rust Code Style

Follow the project's Rust style guidelines:

- Run `just fmt` before committing
- Use `just lint` to check for issues
- Follow patterns in existing code
- Use `tracing` for logging (not `println!`)
- Avoid `.unwrap()` and `.expect()` - use proper error handling

### COSMIC-Specific Guidelines

1. **No Hard-Coded Values**
   - Use `theme::spacing()` for layout spacing
   - Use theme colors via `theme.cosmic()`
   - No hard-coded dimensions or corner radii

2. **Widget Composition**
   - Use libcosmic widgets from `cosmic::widget`
   - Follow existing UI patterns
   - Use symbolic icons (`name-symbolic`)

3. **Error Handling**
   - Use `Result` types and `?` operator
   - Add proper `tracing` logs for errors
   - Provide fallback values where appropriate

4. **Async Operations**
   - Return `Task` for long-running operations
   - Don't block in `update()` method
   - Use `tokio::spawn` for CPU-intensive work

See `CLAUDE.md` for detailed development standards.

## Commit Guidelines

### Pre-Commit Checklist

Before creating a commit, **ALWAYS** run the code-simplifier agent:

```bash
@code-simplifier review the changes we made
```

This ensures:
- Code clarity and consistency
- Removal of redundant patterns
- Better Rust idioms
- Alignment with codebase conventions

**Exception:** Skip only for trivial changes (typo fixes, comments only).

### Commit Message Format

Use conventional commit format:

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

**Examples:**
```
feat(telephony): add SMS message handling
fix(discovery): resolve UDP broadcast issue
docs(diagnostics): update debugging guide
```

### Co-Authoring

If using AI assistance, include:
```
Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
```

## Pull Request Process

### Before Submitting

1. **Run Pre-Commit Checks**
   ```bash
   just check
   just test
   @cosmic-code-reviewer /pre-commit-check
   ```

2. **Update Documentation**
   - Update README if adding features
   - Add/update doc comments
   - Update CHANGELOG if applicable

3. **Test Thoroughly**
   - Test with real devices if possible
   - Verify UI in both light and dark themes
   - Check for memory leaks in long-running tests

### Pull Request Description

Include in your PR description:

```markdown
## Changes
- Brief description of changes

## Testing
- How you tested the changes
- Test devices/configurations used

## Screenshots/Videos
- UI changes should include screenshots
- Complex interactions should include videos

## Checklist
- [ ] Code follows style guidelines
- [ ] Tests pass (`just test`)
- [ ] Lint passes (`just lint`)
- [ ] Documentation updated
- [ ] AI code review completed
```

### Review Process

1. Automated checks must pass (CI/CD when available)
2. Code review by maintainers
3. Testing on real hardware if applicable
4. Approval from at least one maintainer

## Testing

### Unit Tests

Write unit tests for plugins and core functionality:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_functionality() {
        // Test implementation
    }
}
```

### Integration Tests

For daemon and protocol testing, use integration tests in `tests/` directory.

### Manual Testing

For UI and hardware integration:
1. Test with Android/iOS KDE Connect apps
2. Test all supported plugins
3. Test pairing and connectivity
4. Test in both light and dark themes

## Documentation

### Code Documentation

- Add doc comments to public APIs
- Include usage examples in doc comments
- Document error conditions
- Update module-level documentation

### User Documentation

- Update README for user-facing features
- Update docs/ directory for detailed guides
- Include screenshots for UI features
- Document configuration options

### Debug Documentation

See `docs/DEBUGGING.md` for:
- Diagnostic commands
- Log analysis
- Troubleshooting procedures
- Performance metrics

## Plugin Development

When adding new plugins:

1. Create plugin in `cosmic-connect-protocol/src/plugins/`
2. Implement `Plugin` and `PluginFactory` traits
3. Add config flag in `cosmic-connect-daemon/src/config.rs`
4. Register factory in `cosmic-connect-daemon/src/main.rs`
5. Follow existing plugin patterns (ping, battery, etc.)
6. Add comprehensive tests
7. Document plugin capabilities

See existing plugins for reference implementation.

## Getting Help

- **Questions**: Open a GitHub Discussion
- **Bugs**: Open a GitHub Issue
- **Security**: See SECURITY.md (if exists)
- **Chat**: Join COSMIC community channels

## Code of Conduct

Be respectful, constructive, and professional. We want to build a welcoming community for all contributors.

## License

By contributing, you agree that your contributions will be licensed under the GPL-3.0 License.

---

**Thank you for contributing to COSMIC Connect!** 🚀

Your contributions help make device connectivity on COSMIC Desktop better for everyone.
