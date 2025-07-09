# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

vcs2git is a Rust command-line tool that converts VCS `.repos` files (YAML format commonly used in ROS/Autoware projects) to Git submodules. The tool reads repository definitions from a YAML file and adds them as submodules to the current Git repository.

## Development Commands

### Build
```bash
cargo build           # Debug build
cargo build --release # Release build
```

### Run
```bash
cargo run -- <repo-file> <prefix> [options]  # Run from source
```

### Format and Lint
```bash
cargo fmt      # Format code using rustfmt
cargo clippy   # Run linter
```

### Publish to crates.io
```bash
cargo publish  # Requires authentication
```

## Architecture

The codebase consists of two main modules:

1. **src/main.rs** - CLI entry point and core logic
   - Parses command-line arguments using clap
   - Manages Git repository operations via git2
   - Handles submodule addition, updating, and checkout logic
   - Key functions:
     - `main()` - Entry point and orchestration
     - `checkout_to_version()` - Handles version/branch/tag checkout
     - `fetch()` - Fetches remote references with SSH key authentication

2. **src/vcs.rs** - Data structures for VCS repos files
   - Defines YAML schema for `.repos` files
   - Uses serde for serialization/deserialization
   - Key types:
     - `ReposFile` - Top-level container with repository map
     - `Repo` - Individual repository definition (type, url, version)
     - `RepoType` - Currently only supports Git

## Key Implementation Details

- Uses `git2` library for all Git operations
- SSH authentication via SSH agent for private repositories
- Supports selecting/skipping specific repositories via CLI flags
- Can update existing submodules to new versions with `--update` flag
- Handles both branch names and commit hashes for versioning
- Creates parent directories automatically when adding submodules
- Preserves order using `IndexMap` for deterministic processing

## Error Handling

The project uses `anyhow` for error propagation with context messages. All errors bubble up to main() and are displayed with full context chain.