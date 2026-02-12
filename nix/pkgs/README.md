# nixpkgs Package for COSMIC Connect

This directory contains the nixpkgs-compatible package definition for cosmic-connect, intended for submission to the official nixpkgs repository.

## Package Structure

```
nix/pkgs/
└── cosmic-connect.nix    # nixpkgs-compatible package derivation
```

## Differences from `nix/package.nix`

The package in this directory (`cosmic-connect.nix`) differs from `../package.nix` in several key ways to meet nixpkgs requirements:

### Source Handling

**Flake version (`../package.nix`):**
```nix
src = lib.cleanSourceWith {
  src = ../.;
  # Excludes git submodules, uses local source
};
```

**nixpkgs version (`cosmic-connect.nix`):**
```nix
src = fetchFromGitHub {
  owner = "olafkfreund";
  repo = "cosmic-connect-desktop-app";
  rev = "v${version}";
  hash = "sha256-...";  # Fixed hash required
};
```

### Cargo Dependencies

**Flake version:**
```nix
cargoLock = {
  lockFile = ../Cargo.lock;
  allowBuiltinFetchGit = true;  # Acceptable for external flakes
};
```

**nixpkgs version:**
```nix
cargoLock = {
  lockFile = ./Cargo.lock;  # Must be in same directory
  outputHashes = {
    "cosmic-ext-connect-core-0.1.0" = "sha256-...";  # Manual hash required
  };
};
```

### Desktop Files

**Flake version:**
- Generates desktop files inline in `postInstall`
- Useful for rapid development

**nixpkgs version:**
- Uses actual desktop files from source tree
- Follows nixpkgs convention of using upstream files
- Ensures consistency with source

### File Paths

**Flake version:**
- Paths relative to repository root (`../.`)
- Works within flake sandbox

**nixpkgs version:**
- Paths relative to package directory (`./`)
- Designed for `pkgs/by-name/co/cosmic-connect/` structure

## Preparing for nixpkgs Submission

### Required Steps

1. **Tag a Release**
   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```

2. **Get Source Hash**
   ```bash
   nix-prefetch-url --unpack \
     https://github.com/olafkfreund/cosmic-connect-desktop-app/archive/refs/tags/v0.1.0.tar.gz
   ```

3. **Get Dependency Hash**
   ```bash
   # Find cosmic-ext-connect-core commit from Cargo.lock
   grep -A 5 "cosmic-ext-connect-core" ../../Cargo.lock

   # Get hash for that specific commit
   nix-prefetch-git https://github.com/olafkfreund/cosmic-ext-connect-core.git \
     --rev <COMMIT_HASH>
   ```

4. **Update cosmic-connect.nix**
   - Fill in `src.hash` with source hash from step 2
   - Fill in `outputHashes."cosmic-ext-connect-core-0.1.0"` with hash from step 3
   - Add your maintainer information

5. **Test Build**
   ```bash
   # From repository root
   nix-build -E 'with import <nixpkgs> {}; callPackage ./nix/pkgs/cosmic-connect.nix {}'
   ```

### nixpkgs Directory Structure

When submitting to nixpkgs, the package will be placed in:

```
pkgs/by-name/co/cosmic-connect/
├── package.nix           # This file (renamed from cosmic-connect.nix)
└── Cargo.lock            # Copy from repository root
```

Note: In nixpkgs `by-name` structure, the derivation must be named `package.nix`.

## Testing the Package

### Local Build Test

```bash
# Build the package
nix-build -E 'with import <nixpkgs> {}; callPackage ./nix/pkgs/cosmic-connect.nix {}'

# Test binaries
./result/bin/cosmic-ext-applet-connect --version
./result/bin/cosmic-ext-connect-daemon --version
./result/bin/cosmic-ext-connect-manager --version
./result/bin/cosmic-ext-messages-popup --version
```

### NixOS Module Test

See `../module.nix` for the NixOS module. Test with:

```bash
nixos-rebuild build-vm -I nixos-config=./test.nix
```

Where `test.nix`:
```nix
{ config, pkgs, ... }:
{
  imports = [ <nixpkgs/nixos/modules/installer/cd-dvd/installation-cd-minimal.nix> ];

  nixpkgs.overlays = [
    (final: prev: {
      cosmic-connect = prev.callPackage ./nix/pkgs/cosmic-connect.nix {};
    })
  ];

  services.cosmic-connect = {
    enable = true;
    daemon.enable = true;
  };
}
```

## Submission Checklist

Before submitting to nixpkgs:

- [ ] Source hash is correct and verified
- [ ] All git dependency hashes are correct
- [ ] Package builds successfully
- [ ] All binaries execute without errors
- [ ] Desktop files are installed correctly
- [ ] Systemd service is properly configured
- [ ] DBus service file is correct
- [ ] Maintainer information is added
- [ ] License matches source (GPL-3.0-or-later)
- [ ] Meta description is accurate
- [ ] All dependencies are available in nixpkgs
- [ ] Tested on x86_64-linux
- [ ] (Optional) Tested on aarch64-linux

## Maintenance

### Updating the Package

When releasing a new version:

1. Update `version` in `cosmic-connect.nix`
2. Get new source hash
3. Update git dependency hashes if Cargo.lock changed
4. Test build
5. Submit PR to nixpkgs

### Handling Build Failures

Common issues:

**Missing dependency hash:**
```
error: hash mismatch for cosmic-ext-connect-core
```
Solution: Run `nix-prefetch-git` for the specific commit

**Wrong source hash:**
```
error: hash mismatch in fixed-output derivation
```
Solution: Re-run `nix-prefetch-url` with correct tag

**Build failures:**
- Check that all dependencies are in nixpkgs
- Verify feature flags are correct
- Ensure system dependencies are listed

## Resources

- [Issue #43: Submit Package to nixpkgs](https://github.com/olafkfreund/cosmic-connect-desktop-app/issues/43)
- [CONTRIBUTING.md](../../CONTRIBUTING.md) - See "Submitting to nixpkgs" section
- [nixpkgs Contributing Guide](https://github.com/NixOS/nixpkgs/blob/master/CONTRIBUTING.md)
- [Rust Packages in nixpkgs](https://nixos.org/manual/nixpkgs/stable/#rust)

## Contact

For questions about the nixpkgs package:

- Open an issue in this repository
- Ask in the nixpkgs PR once submitted
- Join NixOS community channels

---

**Status:** Ready for submission after hashes are filled in and tested
**Target:** nixpkgs-unstable
**Platforms:** x86_64-linux, aarch64-linux
