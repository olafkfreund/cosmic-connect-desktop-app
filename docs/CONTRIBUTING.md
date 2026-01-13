# Contributing to COSMIC KDE Connect

Thank you for your interest in contributing to COSMIC KDE Connect! This guide will help you get started with development, testing, and submitting contributions.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Environment](#development-environment)
- [Project Structure](#project-structure)
- [Development Workflow](#development-workflow)
- [Coding Standards](#coding-standards)
- [Testing](#testing)
- [Documentation](#documentation)
- [Submitting Changes](#submitting-changes)
- [Issue Guidelines](#issue-guidelines)
- [Pull Request Process](#pull-request-process)
- [Release Process](#release-process)

## Code of Conduct

This project follows the [COSMIC Desktop Code of Conduct](https://github.com/pop-os/cosmic-comp/blob/master/CODE_OF_CONDUCT.md). By participating, you agree to uphold this code.

### Our Standards

- Be respectful and inclusive
- Welcome newcomers and help them learn
- Focus on constructive feedback
- Prioritize the community's best interests
- Show empathy toward others

## Getting Started

### Prerequisites

Before contributing, ensure you have:

- Rust 1.70 or later
- Just command runner
- Git for version control
- A GitHub account
- Familiarity with Rust and async programming

### First-Time Contributors

1. **Star the repository** to show support
2. **Read the documentation** in the `docs/` folder
3. **Check open issues** for beginner-friendly tasks (labeled `good first issue`)
4. **Join the community** on [COSMIC Chat](https://chat.pop-os.org/)

### Finding Issues to Work On

Issues are labeled to help you find suitable tasks:

- `good first issue` - Perfect for newcomers
- `help wanted` - Community contributions welcome
- `bug` - Something isn't working
- `enhancement` - New feature or improvement
- `documentation` - Documentation improvements
- `testing` - Testing-related tasks

## Development Environment

### NixOS (Recommended)

```bash
# Clone the repository
git clone https://github.com/olafkfreund/cosmic-applet-kdeconnect.git
cd cosmic-applet-kdeconnect

# Enter development shell
nix develop

# Build the project
just build

# Run tests
just test
```

### Other Linux Distributions

```bash
# Install system dependencies (Ubuntu/Debian)
sudo apt install \
  build-essential \
  pkg-config \
  libxkbcommon-dev \
  libwayland-dev \
  libdbus-1-dev \
  libssl-dev \
  libfontconfig-dev \
  libfreetype-dev

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Just
cargo install just

# Clone and build
git clone https://github.com/olafkfreund/cosmic-applet-kdeconnect.git
cd cosmic-applet-kdeconnect
just build
```

### Development Tools

We recommend these tools for development:

- **rust-analyzer** - LSP for Rust
- **cargo-watch** - Auto-rebuild on changes
- **cargo-edit** - Manage dependencies
- **cargo-audit** - Security vulnerability scanning

```bash
# Install development tools
cargo install cargo-watch cargo-edit cargo-audit
```

## Project Structure

```
cosmic-applet-kdeconnect/
├── .github/
│   ├── workflows/         # CI/CD pipelines
│   └── dependabot.yml     # Dependency updates
├── cosmic-applet-kdeconnect/  # Panel applet
│   ├── src/
│   │   └── main.rs        # Applet implementation
│   └── Cargo.toml
├── cosmic-kdeconnect/     # Full application (future)
│   └── src/
├── kdeconnect-daemon/     # Background service
│   ├── src/
│   │   └── main.rs        # Daemon implementation
│   └── Cargo.toml
├── kdeconnect-protocol/   # Core protocol library
│   ├── src/
│   │   ├── lib.rs         # Public API
│   │   ├── discovery.rs   # Device discovery
│   │   ├── pairing.rs     # TLS pairing
│   │   ├── packet.rs      # Packet serialization
│   │   ├── device.rs      # Device management
│   │   ├── transport/     # Network/Bluetooth
│   │   └── plugins/       # Plugin implementations
│   ├── tests/
│   │   ├── integration_tests.rs
│   │   └── *.rs
│   └── Cargo.toml
├── docs/                  # Documentation
│   ├── INSTALL.md
│   ├── USER_GUIDE.md
│   ├── TROUBLESHOOTING.md
│   └── CONTRIBUTING.md
├── hooks/                 # Git hooks
│   ├── pre-commit
│   └── commit-msg
├── justfile              # Build commands
├── Cargo.toml            # Workspace config
└── README.md
```

### Crate Responsibilities

- **kdeconnect-protocol**: Core protocol logic, device management, plugins
- **kdeconnect-daemon**: Background service, device discovery, connection handling
- **cosmic-applet-kdeconnect**: UI applet for COSMIC panel
- **cosmic-kdeconnect**: Full application (planned)

## Development Workflow

### Setting Up Your Fork

```bash
# Fork the repository on GitHub
# Then clone your fork
git clone https://github.com/YOUR_USERNAME/cosmic-applet-kdeconnect.git
cd cosmic-applet-kdeconnect

# Add upstream remote
git remote add upstream https://github.com/olafkfreund/cosmic-applet-kdeconnect.git

# Verify remotes
git remote -v
```

### Creating a Feature Branch

```bash
# Update main
git checkout main
git pull upstream main

# Create feature branch
git checkout -b feature/your-feature-name

# Or for bug fixes
git checkout -b fix/issue-description
```

### Branch Naming Convention

- `feature/` - New features
- `fix/` - Bug fixes
- `docs/` - Documentation changes
- `refactor/` - Code refactoring
- `test/` - Adding tests
- `chore/` - Maintenance tasks

### Installing Git Hooks

We provide pre-commit hooks for code quality:

```bash
# Install hooks
just setup

# Or manually
just install-hooks
```

Hooks automatically:
- Format code with `cargo fmt`
- Run linting with `cargo clippy`
- Run tests with `cargo test`
- Enforce commit message format

### Making Changes

1. **Write Tests First** (TDD approach)
   ```bash
   # Add test for new functionality
   cargo test --package kdeconnect-protocol -- your_test --exact
   ```

2. **Implement Feature**
   ```bash
   # Use cargo-watch for auto-rebuild
   cargo watch -x 'build --package kdeconnect-protocol'
   ```

3. **Run Full Test Suite**
   ```bash
   just test
   ```

4. **Check Code Quality**
   ```bash
   just lint
   just fmt
   ```

### Common Just Commands

```bash
# Build all packages
just build

# Build in release mode
just build-release

# Run tests
just test

# Verbose test output
just test-verbose

# Format code
just fmt

# Lint code
just lint

# Security audit
just audit

# Run specific package tests
just test-protocol
just test-daemon
just test-applet

# Clean build artifacts
just clean

# Install locally
just install-local
```

## Coding Standards

### Rust Style Guide

Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/):

1. **Naming Conventions**
   ```rust
   // Modules: snake_case
   mod device_manager;

   // Structs/Enums: PascalCase
   struct DeviceInfo;
   enum ConnectionState;

   // Functions/methods: snake_case
   fn connect_to_device() {}

   // Constants: SCREAMING_SNAKE_CASE
   const MAX_RETRY_COUNT: u32 = 5;

   // Type parameters: PascalCase
   fn generic_function<T: Trait>() {}
   ```

2. **Error Handling**
   ```rust
   // Use Result for fallible operations
   fn parse_packet(data: &[u8]) -> Result<Packet, PacketError> {
       // Implementation
   }

   // Use ? operator for error propagation
   let device = Device::from_discovery(info)?;

   // Provide context with .context() or .map_err()
   fs::read_to_string(path)
       .context("Failed to read configuration file")?;
   ```

3. **Documentation**
   ```rust
   /// Discovers devices on the local network.
   ///
   /// This function broadcasts a UDP packet on port 1716 and listens
   /// for responses from KDE Connect devices.
   ///
   /// # Arguments
   ///
   /// * `timeout` - Maximum time to wait for responses
   ///
   /// # Returns
   ///
   /// A vector of discovered devices.
   ///
   /// # Errors
   ///
   /// Returns `DiscoveryError` if the network is unavailable.
   ///
   /// # Examples
   ///
   /// ```
   /// use kdeconnect_protocol::discover_devices;
   /// use std::time::Duration;
   ///
   /// let devices = discover_devices(Duration::from_secs(5))?;
   /// for device in devices {
   ///     println!("Found: {}", device.name);
   /// }
   /// ```
   pub async fn discover_devices(timeout: Duration) -> Result<Vec<Device>, DiscoveryError> {
       // Implementation
   }
   ```

4. **Async Code**
   ```rust
   // Use async/await for I/O operations
   pub async fn send_packet(&self, packet: Packet) -> Result<(), NetworkError> {
       let data = packet.to_bytes()?;
       self.stream.write_all(&data).await?;
       Ok(())
   }

   // Use tokio::spawn for concurrent tasks
   tokio::spawn(async move {
       // Background task
   });

   // Use proper error handling in async contexts
   async fn handle_connection(stream: TcpStream) -> Result<(), Box<dyn Error>> {
       // Implementation
   }
   ```

5. **Module Organization**
   ```rust
   // Re-export public types from lib.rs
   pub use device::{Device, DeviceInfo, DeviceManager};
   pub use pairing::{PairingHandler, PairingStatus};

   // Use privacy levels appropriately
   pub struct Device { /* public fields */ }
   pub(crate) struct InternalState { /* crate-visible */ }
   struct PrivateData { /* private */ }
   ```

### libcosmic Patterns

When working with the COSMIC applet:

```rust
// Use Task instead of Command
fn update(&mut self, message: Message) -> Task<Message> {
    match message {
        Message::Action => {
            // Perform action
            Task::none()
        }
    }
}

// Use proper lifetime annotations
fn view(&self) -> Element<'_, Message> {
    // UI code
}

// Use surface actions for popups
Message::OpenPopup => {
    return cosmic::task::message(cosmic::Action::Cosmic(
        cosmic::app::Action::Surface(app_popup::<App>(
            // Popup configuration
        ))
    ));
}
```

### Code Quality Checklist

Before submitting:

- [ ] Code follows Rust style guidelines
- [ ] All public APIs are documented
- [ ] Tests added for new functionality
- [ ] Existing tests pass
- [ ] No compiler warnings
- [ ] `cargo clippy` passes
- [ ] `cargo fmt` applied
- [ ] No new security vulnerabilities (`cargo audit`)

## Testing

### Test Organization

```
kdeconnect-protocol/
├── src/
│   └── *.rs           # Unit tests in same file
└── tests/
    └── *.rs           # Integration tests
```

### Writing Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_creation() {
        let info = DeviceInfo::new("Test", DeviceType::Phone, 1716);
        let device = Device::from_discovery(info);
        assert_eq!(device.info.device_name, "Test");
    }

    #[tokio::test]
    async fn test_async_function() {
        let result = some_async_function().await;
        assert!(result.is_ok());
    }
}
```

### Writing Integration Tests

```rust
// tests/integration_tests.rs
use kdeconnect_protocol::{Device, DeviceManager};
use tempfile::TempDir;

fn create_test_manager() -> DeviceManager {
    let temp_dir = TempDir::new().unwrap();
    let registry_path = temp_dir.path().join("registry.json");
    DeviceManager::new(registry_path).unwrap()
}

#[tokio::test]
async fn test_device_pairing() {
    let mut manager = create_test_manager();
    // Test implementation
}
```

### Test Categories

1. **Unit Tests**: Test individual functions/methods
2. **Integration Tests**: Test component interactions
3. **Protocol Tests**: Test protocol compliance
4. **UI Tests**: Test applet functionality (manual for now)

### Running Tests

```bash
# All tests
just test

# Specific package
cargo test -p kdeconnect-protocol

# Specific test
cargo test test_device_creation

# With output
just test-verbose

# Watch mode
cargo watch -x test
```

### Test Coverage

We use `cargo-llvm-cov` for coverage:

```bash
# Install
cargo install cargo-llvm-cov

# Generate coverage
cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info

# View HTML report
cargo llvm-cov --all-features --workspace --html
open target/llvm-cov/html/index.html
```

## Documentation

### Code Documentation

- Document all public APIs with `///`
- Include examples in doc comments
- Explain why, not just what
- Link to related functions with `[`function_name`]`

### User Documentation

Located in `docs/`:

- `INSTALL.md` - Installation instructions
- `USER_GUIDE.md` - Usage guide
- `TROUBLESHOOTING.md` - Common issues
- `CONTRIBUTING.md` - This file

### Updating Documentation

When adding features:

1. Update relevant doc comments
2. Update user documentation
3. Add examples if applicable
4. Update README.md if needed

## Submitting Changes

### Commit Message Format

We follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types**:
- `feat` - New feature
- `fix` - Bug fix
- `docs` - Documentation only
- `style` - Code style (formatting, etc.)
- `refactor` - Code refactoring
- `test` - Adding tests
- `chore` - Maintenance

**Scopes**:
- `protocol` - Protocol library
- `daemon` - Daemon service
- `applet` - COSMIC applet
- `ci` - CI/CD
- `deps` - Dependencies

**Examples**:

```
feat(protocol): add Bluetooth transport support

Implement Bluetooth transport alongside existing TCP transport.
Devices can now connect via Bluetooth when WiFi is unavailable.

Closes #42
```

```
fix(applet): correct device status indicator color

The status indicator was showing green for disconnected devices.
Changed to use ConnectionState enum properly.

Fixes #156
```

```
docs: update installation guide for Fedora 40

Add specific instructions for Fedora 40 dependencies.
Update firewall configuration commands.
```

### Pre-Commit Checklist

- [ ] All tests pass
- [ ] Code is formatted (`cargo fmt`)
- [ ] No linter warnings (`cargo clippy`)
- [ ] Documentation updated
- [ ] Commit message follows convention
- [ ] Branch is up to date with main

### Pushing Changes

```bash
# Ensure tests pass
just test

# Format code
just fmt

# Push to your fork
git push origin feature/your-feature

# If you need to update your branch
git fetch upstream
git rebase upstream/main
git push origin feature/your-feature --force-with-lease
```

## Issue Guidelines

### Creating Issues

**Bug Reports** should include:

```markdown
**Description**
Clear description of the bug.

**Steps to Reproduce**
1. Step one
2. Step two
3. Expected vs actual result

**Environment**
- OS: [NixOS 24.11]
- Rust version: [1.75]
- Commit/version: [abc123]

**Logs**
```
Relevant log output
```

**Additional Context**
Any other relevant information.
```

**Feature Requests** should include:

```markdown
**Problem Statement**
What problem does this solve?

**Proposed Solution**
How should it work?

**Alternatives Considered**
Other approaches you've thought about.

**Additional Context**
Use cases, examples, etc.
```

### Issue Labels

Maintainers will add labels:

- **Priority**: `P0` (critical) to `P3` (low)
- **Status**: `investigating`, `confirmed`, `blocked`
- **Area**: `protocol`, `daemon`, `applet`, `docs`

## Pull Request Process

### Before Creating a PR

1. **Create/claim an issue** first
2. **Discuss approach** if non-trivial
3. **Ensure tests pass** locally
4. **Update documentation** if needed
5. **Rebase on latest main** to avoid conflicts

### Creating a Pull Request

1. **Push to your fork**
   ```bash
   git push origin feature/your-feature
   ```

2. **Open PR on GitHub**
   - Use descriptive title
   - Fill out PR template
   - Link related issues
   - Request review

### PR Template

```markdown
## Description
Clear description of changes.

## Related Issues
Closes #123
Relates to #456

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Documentation update
- [ ] Refactoring

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] Manual testing completed

## Checklist
- [ ] Code follows project style
- [ ] Documentation updated
- [ ] Tests pass locally
- [ ] No new compiler warnings
- [ ] Commit messages follow convention

## Screenshots (if applicable)
[Add screenshots for UI changes]
```

### Review Process

1. **Automated Checks** run (CI/CD)
2. **Maintainer Review** (usually within 1-2 days)
3. **Address Feedback** if requested
4. **Approval** from maintainer
5. **Merge** to main branch

### After PR is Merged

- **Delete your branch** (optional)
- **Update your fork**
  ```bash
  git checkout main
  git pull upstream main
  git push origin main
  ```

## Release Process

### Version Numbering

We follow [Semantic Versioning](https://semver.org/):

- `MAJOR.MINOR.PATCH`
- `MAJOR` - Breaking changes
- `MINOR` - New features (backward compatible)
- `PATCH` - Bug fixes

### Release Checklist

For maintainers:

1. **Update Cargo.toml** versions
2. **Update CHANGELOG.md**
3. **Create release tag**
   ```bash
   git tag -a v0.2.0 -m "Release v0.2.0"
   git push upstream v0.2.0
   ```
4. **Create GitHub release** with notes
5. **Publish to crates.io** (if applicable)
   ```bash
   cargo publish -p kdeconnect-protocol
   ```

## Development Tips

### Debugging

```bash
# Run with debug output
RUST_LOG=debug cargo run --package kdeconnect-daemon

# Use specific module logging
RUST_LOG=kdeconnect_protocol::discovery=trace cargo run

# Debug with gdb
rust-gdb target/debug/kdeconnect-daemon
```

### Performance Profiling

```bash
# Install flamegraph
cargo install flamegraph

# Generate flamegraph
cargo flamegraph --package kdeconnect-daemon
```

### Useful cargo Commands

```bash
# Check without building
cargo check

# Build documentation
cargo doc --open

# Update dependencies
cargo update

# Show dependency tree
cargo tree

# Analyze binary size
cargo bloat --release
```

## Getting Help

### Resources

- **Documentation**: See `docs/` folder
- **Code Examples**: Check `examples/` (when available)
- **Tests**: Look at existing tests for patterns
- **Issues**: Search closed issues for solutions

### Communication

- **GitHub Discussions**: For questions and ideas
- **GitHub Issues**: For bugs and feature requests
- **COSMIC Chat**: For real-time discussion

### Mentorship

New contributors welcome! Look for:

- `good first issue` label
- `mentorship available` label
- Ask in discussions for guidance

## Recognition

Contributors are recognized:

- In release notes
- In the CONTRIBUTORS file (if we create one)
- By GitHub's contributor graph

## License

By contributing, you agree that your contributions will be licensed under the same license as the project (GPL-3.0).

---

**Thank you for contributing to COSMIC KDE Connect!**

For questions about this guide, open a [GitHub Discussion](https://github.com/olafkfreund/cosmic-applet-kdeconnect/discussions).

**Last Updated**: 2026-01-13
**Version**: 1.0.0
