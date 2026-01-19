---
name: Rust Development Best Practices
description: Guidelines, idioms, and best practices for writing high-quality Rust code.
---

# Rust Development Best Practices

This skill outlines the industry standards and best practices for developing software in Rust.

## 1. Project Layout (The Cargo Standard)

Follow the standard Cargo project layout strictly:

- `src/main.rs`: Entry point for binary crates.
- `src/lib.rs`: Entry point for library crates (and the library part of binaries).
- `src/bin/`: Additional binaries.
- `tests/`: Integration tests (treats your crate as an external dependency).
- `benches/`: Benchmarks (using `criterion` usually).
- `examples/`: Example usage code.

### Guidelines

- **Split `main.rs` and `lib.rs`**: Even for binaries, put most logic in `lib.rs`. `main.rs` should only parse args and call into `lib.rs`. This makes logic testable.
- **Workspaces**: For multi-crate projects, use a [Cargo Workspace](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html) in the root `Cargo.toml`.

## 2. Idiomatic Rust

- **Clippy is Law**: Always run `cargo clippy`. Fix all warnings. It catches non-idiomatic code.
- **Result Handling**:
  - Never use `.unwrap()` or `.expect()` in production/library code (recoverable errors).
  - Use `?` operator for error propagation.
  - Use `.expect("...")` only in tests or prototypes to document _why_ it shouldn't fail.
- **Iterators**: Prefer iterator chains (`.map()`, `.filter()`, `.collect()`) over raw `for` loops for transformation logic. It's often faster and more readable.
- **Type Safety**: Use the type system!
  - **Newtypes**: `struct UserId(String)` instead of passing raw strings.
  - **Enums**: Make invalid states unrepresentable.

## 3. Error Handling

- **Libraries**: Define a custom `Error` enum using `thiserror`.
  ```rust
  #[derive(thiserror::Error, Debug)]
  pub enum MyError {
      #[error("IO error: {0}")]
      Io(#[from] std::io::Error),
      #[error("Invalid input: {0}")]
      InvalidInput(String),
  }
  ```
- **Binaries/Apps**: Use `anyhow::Result` for flexible error bubbling in applications.
  ```rust
  fn main() -> anyhow::Result<()> { ... }
  ```

## 4. Async Rust (Tokio)

- **Runtime**: Use `#[tokio::main]` for the entry point.
- **Blocking**: NEVER call blocking functions (file IO, heavy computation, `std::thread::sleep`) inside an async function. It blocks the executor thread.
  - Use `tokio::fs` instead of `std::fs`.
  - Use `tokio::task::spawn_blocking` for CPU-heavy or blocking synchronous APIs.
- **Concurrency**:
  - `tokio::spawn`: Spawns a background task (fire and forget or await handle).
  - `tokio::join!`: Wait for multiple futures concurrently.
  - `tokio::select!`: Race multiple futures (cancellation safety is key here).
- **Channels**: Use `tokio::sync::mpsc` for message passing between tasks.

## 5. Testing

- **Unit Tests**: Put them in the same file as the code, in a `mod tests` block.
  ```rust
  #[cfg(test)]
  mod tests {
      use super::*;
      #[test]
      fn it_works() { ... }
  }
  ```
- **Integration Tests**: Put them in `tests/*.rs`. These test your public API.
- **Property Testing**: Consider `proptest` for complex logic to automatically find edge cases.

## 6. Formatting & Documentation

- **Formatting**: Always run `cargo fmt`. No arguments.
- **Docs**:
  - Triple slash `///` for public item documentation.
  - Module level docs `//!` at the top of files.
  - Run `cargo doc --open` to preview.

## 7. Common Dependencies

- **Serialization**: `serde`, `serde_json`
- **Async**: `tokio`, `futures`
- **Error**: `thiserror` (lib), `anyhow` (bin)
- **Logging**: `tracing`, `tracing-subscriber` (do not use `log` crate directly anymore, `tracing` is the standard for async apps).
- **CLI**: `clap`
