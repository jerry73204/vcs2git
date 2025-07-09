# Architecture Documentation

## Overview

vcs2git is a command-line tool that bridges the gap between VCS `.repos` files (commonly used in ROS/Autoware ecosystems) and Git submodules. The tool provides a streamlined way to manage multiple Git repository dependencies by converting YAML-based repository definitions into Git submodules.

**Important**: vcs2git exclusively supports Git repositories. Other version control systems (Mercurial, SVN, Bazaar) are not supported by design, as Git submodules can only reference Git repositories.

## System Architecture

### High-Level Design

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   .repos file   │───▶│    vcs2git       │───▶│ Git submodules  │
│   (YAML input)  │    │  (conversion)    │    │   (output)      │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

### Core Components

#### 1. CLI Interface (`src/main.rs`)
- **Command-line Parsing**: Uses `clap` with derive macros for ergonomic argument handling
- **Repository Management**: Orchestrates the entire conversion process
- **Git Operations**: Manages submodule lifecycle (add, update, checkout)

#### 2. Data Model (`src/vcs.rs`)
- **YAML Schema**: Defines the structure for `.repos` files
- **Type Safety**: Strongly-typed representation of repository definitions
- **Serialization**: Handles YAML parsing and validation

### Data Flow

```
1. Parse CLI arguments
2. Open current Git repository
3. List existing submodules
4. Parse .repos YAML file
5. Filter repositories (select/skip logic)
6. Validate submodule states:
   - Check for modified content in existing submodules
   - Verify submodules are initialized and not deinitialized
   - Ensure working trees are clean (no uncommitted changes)
7. Capture original submodule states:
   - Record commit SHA for each existing submodule
   - Store submodule names, paths, and URLs
8. Categorize repositories:
   - New repositories (to be added)
   - Existing submodules (to be updated if --update)
   - Extra submodules (existing but not in repos file)
9. Process repositories with rollback support:
   - Add new submodules or update existing ones
   - On any failure:
     * Remove newly added submodules
     * Restore all submodules to original commits
     * Report rollback status to user
10. Complete successfully or restore original state
```

### Key Design Patterns

#### Error Handling Strategy
- **Centralized Error Management**: Uses `anyhow::Result` for consistent error propagation
- **Context Enrichment**: Adds meaningful context at each layer
- **Fail-Fast Approach**: Validates inputs early and stops on first error

#### Repository State Management
- **State Tracking**: Captures submodule commit SHAs before any modifications
- **Atomic Operations**: All-or-nothing approach with complete rollback on failure
- **Clean State Requirement**: Validates no uncommitted changes before operations
- **Commit-Based Recovery**: Restores exact commits rather than branches on rollback

#### Authentication Handling
- **SSH Agent Integration**: Leverages system SSH agent for authentication
- **Credential Delegation**: Relies on Git's existing credential management

## Technology Stack

### Core Dependencies

| Library                | Purpose                    | Version          |
|------------------------|----------------------------|------------------|
| `clap`                 | CLI argument parsing       | 4.5.1            |
| `git2`                 | Git operations via libgit2 | 0.18.2           |
| `serde` + `serde_yaml` | YAML serialization         | 1.0.196 + 0.9.31 |
| `anyhow`               | Error handling             | 1.0.79           |
| `indexmap`             | Ordered hash maps          | 2.2.3            |
| `url`                  | URL parsing and validation | 2.5.0            |

### Design Rationale

#### Choice of Rust
- **Memory Safety**: Eliminates common classes of bugs
- **Performance**: Near-native performance for Git operations
- **Ecosystem**: Rich ecosystem for CLI tools and Git integration
- **Reliability**: Strong type system prevents runtime errors

#### Git2 Library Selection
- **Native Integration**: Direct binding to libgit2 C library
- **Feature Completeness**: Comprehensive Git operation support
- **Authentication**: Built-in SSH and credential support
- **Cross-Platform**: Works consistently across operating systems

## Scalability Considerations

### Performance Characteristics
- **Linear Complexity**: Processing time scales linearly with repository count
- **Parallel Potential**: Repository operations could be parallelized in future versions
- **Memory Usage**: Minimal memory footprint, processes repositories sequentially

### Limits and Constraints
- **Repository Type**: Only Git repositories are supported (no Mercurial, SVN, or Bazaar)
- **Repository Count**: No theoretical limit, bounded by system resources
- **File Size**: YAML parsing handles reasonably large `.repos` files
- **Network Operations**: Limited by Git protocol and network bandwidth
- **Atomicity**: No transactional guarantees - Git does not support atomic multi-submodule operations
- **Submodule State**: Existing submodules must be in clean state (no modifications, fully initialized)

## Security Model

### Authentication
- **SSH Key-based**: Relies on SSH agent for private repository access
- **No Credential Storage**: Never stores or caches authentication credentials
- **System Integration**: Uses existing Git credential helpers

### Input Validation
- **YAML Schema Validation**: Validates repository definitions against expected schema
- **URL Validation**: Ensures repository URLs are well-formed
- **Path Sanitization**: Validates submodule paths to prevent directory traversal
- **Submodule State Validation**: Ensures existing submodules are in clean state before modification

### Trust Model
- **Repository Trust**: Assumes repository URLs are trusted (same as manual git clone)
- **Version Pinning**: Supports specific commit hashes for reproducible builds
- **No Code Execution**: Tool only performs Git operations, no arbitrary code execution
