# Architectural Decision Records (ADRs)

## ADR-001: Use Rust as Primary Language

**Status**: Accepted
**Date**: 2024
**Deciders**: Project maintainer

### Context
Need to choose a programming language for a tool that performs Git operations and file I/O with reliability and performance requirements.

### Decision
Use Rust as the primary programming language.

### Rationale
- **Memory Safety**: Eliminates entire classes of bugs (buffer overflows, use-after-free)
- **Performance**: Near C-level performance for file and Git operations
- **Ecosystem**: Excellent libraries for CLI tools (`clap`) and Git integration (`git2`)
- **Error Handling**: Built-in Result type encourages proper error handling
- **Cross-platform**: Single codebase works across Linux, macOS, and Windows

### Consequences
- **Positive**: Reliable, fast execution; excellent error messages; strong type safety
- **Negative**: Steeper learning curve for contributors; longer compile times during development

---

## ADR-002: Use git2 Library for Git Operations

**Status**: Accepted
**Date**: 2024
**Deciders**: Project maintainer

### Context
Need to perform Git operations (submodule management, fetching, checkout) programmatically.

### Alternatives Considered
1. **Shell out to git commands**: Simple but fragile, platform-dependent
2. **git2 Rust bindings**: Native library integration
3. **GitOxide**: Pure Rust Git implementation (experimental)

### Decision
Use `git2` library (Rust bindings to libgit2).

### Rationale
- **Mature**: libgit2 is battle-tested and widely used
- **Feature Complete**: Supports all required Git operations
- **Authentication**: Built-in SSH agent integration
- **Error Handling**: Detailed error information from native Git library
- **Performance**: Direct library calls avoid subprocess overhead

### Consequences
- **Positive**: Reliable Git operations; good error reporting; cross-platform consistency
- **Negative**: Large dependency; potential security updates needed for libgit2

---

## ADR-003: Support Only Git Repository Type

**Status**: Accepted and Final
**Date**: 2024
**Deciders**: Project maintainer

### Context
VCS `.repos` files can specify different repository types (git, hg, svn, bzr), but Git submodules only support Git repositories. This is a fundamental limitation of Git's architecture.

### Decision
**Permanently support only `type: git` repositories**. Other VCS types will be explicitly rejected with clear error messages.

### Rationale
- **Technical Impossibility**: Git submodules can only reference Git repositories - this is a hard constraint
- **Simplicity**: Eliminates complexity of VCS conversion or translation layers
- **Git Ecosystem**: Target audience (ROS/Autoware) increasingly uses Git exclusively
- **Clear Scope**: Users understand exactly what the tool does
- **Maintenance**: Focusing on one VCS allows deeper Git-specific optimizations

### Consequences
- **Positive**:
  - Crystal-clear tool purpose and limitations
  - No false expectations about VCS support
  - Simpler codebase and maintenance
  - Can leverage Git-specific features fully
- **Negative**:
  - Cannot process mixed-VCS `.repos` files
  - Users must convert non-Git repositories beforehand
  - Tool is not a drop-in replacement for vcstool

### Future Considerations
This decision is **final and will not be revisited**. The tool's name (vcs2git) explicitly indicates Git as the target, and the architecture is designed around Git-specific operations.

---

## ADR-004: Use IndexMap for Repository Ordering

**Status**: Accepted
**Date**: 2024
**Deciders**: Project maintainer

### Context
Need to preserve the order of repositories as defined in `.repos` files for deterministic behavior.

### Decision
Use `IndexMap` instead of `HashMap` for repository storage.

### Rationale
- **Deterministic Output**: Same input always produces same order of operations
- **User Expectations**: Repositories processed in the order they appear in YAML
- **Debugging**: Predictable order simplifies troubleshooting
- **Minimal Overhead**: IndexMap has similar performance to HashMap for small collections

### Consequences
- **Positive**: Predictable behavior; easier debugging; better user experience
- **Negative**: Slightly higher memory usage; additional dependency

---

## ADR-005: Fail-Fast Validation Strategy

**Status**: Accepted
**Date**: 2024
**Deciders**: Project maintainer

### Context
Tool needs to validate multiple inputs (CLI args, YAML structure, repository URLs) before performing any Git operations.

### Decision
Validate all inputs upfront and fail immediately on first error.

### Rationale
- **Safety**: Prevents partial state changes that are hard to recover from
- **User Experience**: Fast feedback on configuration errors
- **Simplicity**: Easier to reason about than complex rollback mechanisms
- **Resource Efficiency**: Avoids expensive Git operations on invalid inputs

### Consequences
- **Positive**: No partial failures; clear error messages; fast failure detection
- **Negative**: Cannot process valid repositories when some are invalid

---

## ADR-006: SSH Agent Authentication Only

**Status**: Accepted
**Date**: 2024
**Deciders**: Project maintainer

### Context
Need to authenticate with private Git repositories during clone/fetch operations.

### Alternatives Considered
1. **SSH Agent only**: Delegate to system SSH agent
2. **Multiple auth methods**: Support SSH keys, tokens, username/password
3. **Custom credential storage**: Store credentials in tool configuration

### Decision
Support only SSH agent authentication.

### Rationale
- **Security**: No credential storage in tool; leverages existing SSH setup
- **Simplicity**: Single authentication path reduces complexity
- **User Familiarity**: Developers already use SSH agents for Git operations
- **Integration**: Works seamlessly with existing Git workflows

### Consequences
- **Positive**: Secure; simple implementation; leverages existing user setup
- **Negative**: Requires SSH agent setup; may not work in some CI environments

---

## ADR-007: Atomic Submodule Operations

**Status**: Accepted
**Date**: 2024
**Deciders**: Project maintainer

### Context
Tool performs multiple Git operations per repository (add submodule, fetch, checkout). Need to handle partial failures.

### Decision
Make each repository's operations atomic but allow failure of individual repositories.

### Rationale
- **Partial Success**: Can process other repositories even if one fails
- **User Control**: Users can fix issues and re-run tool
- **Git Safety**: Each submodule is either fully added or not added at all
- **Debugging**: Clear indication of which repositories succeeded/failed

### Consequences
- **Positive**: Robust handling of individual repository failures; clear status reporting
- **Negative**: More complex error handling; may leave repositories in different states

---

## ADR-008: No Built-in Parallelization

**Status**: Accepted
**Date**: 2024
**Deciders**: Project maintainer

### Context
Repository operations (clone, fetch) can be time-consuming and could benefit from parallelization.

### Decision
Process repositories sequentially in current version.

### Rationale
- **Simplicity**: Easier to implement and debug
- **Git Locks**: Avoids potential Git repository locking issues
- **Resource Management**: Prevents overwhelming system resources
- **Error Handling**: Simpler error reporting and recovery
- **Future Enhancement**: Can be added later without breaking changes

### Consequences
- **Positive**: Simple implementation; predictable resource usage; easier debugging
- **Negative**: Slower execution for large repository lists; missed optimization opportunity

---

## ADR-009: Require Clean Submodule State

**Status**: Accepted
**Date**: 2024
**Deciders**: Project maintainer

### Context
When updating existing submodules, the tool needs to handle repositories that may have local modifications, be deinitialized, or be in an inconsistent state.

### Decision
Require all existing submodules to be in a clean state before allowing any modifications. Fail fast if any submodule has uncommitted changes or is deinitialized.

### Rationale
- **Data Safety**: Prevents accidental loss of uncommitted work
- **Predictable Behavior**: Operations work on known-good state
- **Git Best Practices**: Aligns with Git's philosophy of protecting user data
- **Clear Expectations**: Users must explicitly handle their changes before bulk operations
- **Simpler Implementation**: No need to handle complex merge or stash scenarios

### Implementation Requirements
1. Check all submodules are initialized (not deinitialized)
2. Verify no uncommitted changes in submodule working trees
3. Ensure submodule HEAD matches expected commit from superproject
4. Validate main repository has no staged changes that could conflict

### Consequences
- **Positive**:
  - Protects user work from accidental loss
  - Predictable and safe operations
  - Clear error messages guide users to resolve issues
  - Aligns with Git's safety principles
- **Negative**:
  - Users must clean up before running tool
  - Cannot handle repositories with work-in-progress
  - May require multiple steps for users with many dirty submodules

### Error Messages
Provide clear, actionable error messages:
- "Submodule 'X' has uncommitted changes. Please commit or stash changes."
- "Submodule 'Y' is not initialized. Run 'git submodule update --init' first."
- "Repository has staged changes. Please commit or reset before proceeding."

---

## ADR-010: Commit-Based Rollback Strategy

**Status**: Accepted  
**Date**: 2024  
**Deciders**: Project maintainer  

### Context
When operations fail during submodule processing, the repository can be left in a partially modified state. Need a reliable way to restore the repository to its original state if any operation fails.

### Alternatives Considered
1. **Individual operation recovery**: Clean up each failed operation separately
2. **Git command-based cleanup**: Use `git submodule deinit` and `git rm` for each failure
3. **Commit tracking and restoration**: Track original commits and restore on failure
4. **Checkpoint commits**: Create commits after each successful batch

### Decision
Implement a commit-based rollback strategy that tracks original submodule commits before any modifications and restores them if any operation fails.

### Rationale
- **Precision**: Restores exact commit SHAs, not just branch references
- **Completeness**: Handles both new additions and updates uniformly
- **Git-Native**: Uses Git's own checkout mechanisms
- **User Confidence**: Clear all-or-nothing semantics
- **Clean State Requirement**: Ensures no data loss by requiring clean state upfront

### Implementation Details
1. **Pre-operation State Capture**: Record commit SHA, name, path, and URL for each submodule
2. **State Tracker Object**: Maintain original states throughout operation
3. **Atomic Processing**: Process all operations, rollback everything on any failure
4. **Two-Phase Rollback**:
   - Remove any newly added submodules
   - Restore all original submodules to their captured commits

### Example Flow
```
1. Validate clean repository state
2. Capture: submodA @ abc123, submodB @ def456
3. Begin operations:
   - Add submodC ✓
   - Update submodA to xyz789 ✓
   - Add submodD ✗ (fails)
4. Rollback:
   - Remove submodC
   - Restore submodA to abc123
   - Report failure and rollback completion
```

### Consequences
- **Positive**: 
  - Guaranteed consistent state after operations
  - No partial modifications left behind
  - Users can trust the tool won't corrupt their repository
  - Simple mental model: all succeed or all fail
- **Negative**: 
  - Cannot complete partial work (some users might want this)
  - Requires clean state (no dirty working trees)
  - More complex implementation than individual cleanup

### Future Considerations
- Could add `--partial` flag for users who want to keep successful operations
- Consider progress persistence for very large operations
- May need optimization for repositories with hundreds of submodules

---

## ADR-011: CLI Options Redesign for Repository Selection and Synchronization

**Status**: Accepted  
**Date**: 2025-01-13  
**Deciders**: Project maintainer  

### Context
The current CLI options (`--select`, `--skip`, `--update`) have unclear semantics and don't provide a complete solution for repository synchronization. Users need clear options to:
1. Select which repositories to process
2. Control whether existing submodules are updated
3. Remove submodules that should no longer exist

### Current Issues
- `--select` and `--skip` names are ambiguous
- No way to remove submodules not in the repos file
- `--update` flag is counterintuitive (most users expect updates by default)
- Cannot easily achieve "exact synchronization" with repos file

### Decision
Redesign CLI options with clearer names and semantics:

1. **Repository Selection** (mutually exclusive):
   - `--only <repos>`: Process only these listed repositories
   - `--ignore <repos>`: Process all listed repositories except these

2. **Update Behavior**:
   - Default: Update existing submodules to match repos file
   - `--skip-existing`: Don't update existing submodules (previously `--no-update`)

3. **Synchronization**:
   - `--sync-selection`: Remove submodules not in the current selection

### Behavior Matrix

| Command                                                 | Action                          |
|---------------------------------------------------------|---------------------------------|
| `vcs2git repos.yaml prefix`                             | Add new, update existing        |
| `vcs2git repos.yaml prefix --only A B`                  | Process only A and B            |
| `vcs2git repos.yaml prefix --ignore C`                  | Process all except C            |
| `vcs2git repos.yaml prefix --skip-existing`             | Add new only                    |
| `vcs2git repos.yaml prefix --sync-selection`            | Add/update all, remove unlisted |
| `vcs2git repos.yaml prefix --only A B --sync-selection` | Keep only A and B               |

### Rationale
- **Clarity**: `--only` and `--ignore` clearly indicate selection behavior
- **Intuitive Defaults**: Updating by default matches user expectations
- **Unified Sync**: One flag handles all removal scenarios based on selection
- **Flexibility**: Can achieve any desired end state with combinations

### Migration Plan
1. Deprecate `--select`, `--skip`, `--update` in v0.4.0
2. Show clear migration messages when old flags are used
3. Remove old flags in v1.0.0

### Consequences
- **Positive**:
  - Clearer mental model for users
  - More powerful synchronization capabilities
  - Better default behavior
  - Simpler documentation
- **Negative**:
  - Breaking change requiring migration
  - Users must learn new flags
  - More complex implementation initially
