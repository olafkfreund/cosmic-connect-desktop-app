---
name: NixOS Development
description: Best practices for using NixOS, Flakes, and direnv for reproducible development environments.
---

# NixOS Development Best Practices

This skill outlines how to use NixOS, Nix Flakes, and direnv to create robust, reproducible development environments.

## 1. Core Concepts

- **Nix Flakes**: The modern standard for defining Nix projects. They ensure reproducibility by pinning dependencies in a `flake.lock` file.
- **DevShells**: Isolated development environments defined in `flake.nix`. They provide all necessary compilers, tools, and libraries without polluting your global system.
- **Direnv**: A shell extension that automatically loads/unloads environment variables when you enter/exit directories.
- **Nix-Direnv**: A direnv extension that makes loading Nix shells faster and simpler.

## 2. Flake Structure (`flake.nix`)

Every project should have a `flake.nix` in its root. A standard Rust project flake looks like this:

```nix
{
  description = "My Rust Project";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Rust Toolchain
            (rust-bin.stable.latest.default.override {
              extensions = [ "rust-src" "rust-analyzer" ];
            })

            # Common Tools
            pkg-config
            openssl

            # GUI/System Dependencies (Example for Cosmic/Iced)
            wayland
            libxkbcommon
            vulkan-loader
          ];

          # Environment Variables
          shellHook = ''
            export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath [
              pkgs.wayland
              pkgs.libxkbcommon
              pkgs.vulkan-loader
            ]}:$LD_LIBRARY_PATH

            echo "🚀 Development environment loaded!"
          '';
        };
      }
    );
}
```

## 3. Automatic Environment Loading (`direnv`)

To automatically activate the flake when you `cd` into the directory:

1.  **Install Direnv & Nix-Direnv**:
    Add this to your _system_ configuration (`configuration.nix` or `home-manager`):

    ```nix
    programs.direnv = {
      enable = true;
      nix-direnv.enable = true;
    };
    ```

2.  **Create `.envrc`**:
    In your project root, create a file named `.envrc` with this single line:

    ```bash
    use flake
    ```

3.  **Allow**:
    Run `direnv allow` in the terminal.

## 4. Best Practices

- **Pin Everything**: Commit `flake.lock`. This guarantees that every developer uses the EXACT same version of Rust, external libraries, and system tools.
- **Keep Scripts Local**: Don't rely on global tools. If your project needs `just` or `cargo-nextest`, add them to `buildInputs` in `flake.nix`.
- **System Dependencies**: Unlike other OSs, you cannot just "install libraries" globally. You must define them in `buildInputs`. If a binary fails with "library not found", you likely need to add it to `LD_LIBRARY_PATH` in the `shellHook`.
- **Rust Specifics**:
  - Use `fenix` or `rust-overlay` for managing Rust toolchains in Nix.
  - Set `RUST_SRC_PATH` environment variable if `rust-analyzer` complains about missing source code.

## 5. Troubleshooting

- **"Command not found"**: Check if the package is in `buildInputs`. Run `nix flake update` to refresh inputs.
- **IDE Issues**: Launch your IDE (VS Code, etc.) _from the terminal_ inside the project folder (`code .`). This ensures it inherits the `direnv` environment variables.
- **Slow Loading**: Ensure `nix-direnv` is enabled. It caches the shell calculation.
