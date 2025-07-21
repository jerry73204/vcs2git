# Project Roadmap

## Current Status (v0.3.0)

vcs2git has reached a stable foundation with core functionality for converting VCS `.repos` files to Git submodules. The tool successfully handles basic use cases in the ROS/Autoware ecosystem.

### Completed Features
- ✅ Basic `.repos` to Git submodules conversion
- ✅ Support for Git repositories only
- ✅ SSH agent authentication
- ✅ Selective repository processing (--select, --skip)
- ✅ Existing submodule updates (--update flag)
- ✅ Version/branch/tag checkout support
- ✅ Published on crates.io

---

## Phase 1: Critical Bug Fixes (v0.3.1 - Immediate) ✅ COMPLETED

**Goal**: Fix critical bugs that affect core functionality

### Action Items

| Task                           | Description                                                       | Status       | Priority |
|--------------------------------|-------------------------------------------------------------------|--------------|----------|
| Fix disjoint check logic       | Fix inverted logic in `check_disjoint` function (src/main.rs:225) | ✅ COMPLETED | Critical |
| Fix error messages             | Correct "Repository type 'X' is supported" message                | ✅ COMPLETED | High     |
| Add submodule state validation | Implement clean state checks per ADR-009                          | ✅ COMPLETED | High     |
| Add input validation           | Validate duplicate names, paths, and URLs                         | ✅ COMPLETED | High     |
| Implement state tracking       | Add SubmoduleStateTracker for rollback (per ADR-010)              | ✅ COMPLETED | High     |

### Testing Requirements

| Test Type                      | Description                                                       | Status       | Priority |
|--------------------------------|-------------------------------------------------------------------|--------------|----------|
| Unit tests for validation      | Test check_disjoint, check_subset, validate_repositories          | ✅ COMPLETED | Critical |
| Unit tests for state tracking  | Test SubmoduleStateTracker capture and rollback                  | ✅ COMPLETED | High     |
| Error message tests            | Verify all error messages are correct and helpful                 | ✅ COMPLETED | High     |
| Integration test framework     | Set up test harness with temporary Git repos                      | ✅ COMPLETED | High     |
| Basic smoke tests              | End-to-end test for simple add/update scenarios                   | ✅ COMPLETED | High     |

### Deliverables ✅
- ✅ Bug-free core functionality
- ✅ Clear error messages
- ✅ 80%+ test coverage for critical paths
- ✅ Test infrastructure for future phases

---

## Phase 2: Error Handling & Recovery (v0.4.0 - Q1 2025)

**Goal**: Improve error handling, add recovery mechanisms, and redesign CLI options

### Action Items

| Task                  | Description                                                             | Status  | Priority |
|-----------------------|-------------------------------------------------------------------------|---------|----------|
| CLI options redesign  | Implement --only, --ignore, --skip-existing, --sync-selection (ADR-011) | 🔴 TODO | Critical |
| Deprecation warnings  | Add warnings for old --select, --skip, --update flags                   | 🔴 TODO | High     |
| Sync implementation   | Add submodule removal logic for --sync-selection                        | 🔴 TODO | High     |
| Commit-based rollback | Track submodule commits and restore on failure (per ADR-010)            | 🔴 TODO | High     |
| Atomic operations     | All-or-nothing processing with complete rollback                        | 🔴 TODO | High     |
| Pre-flight validation | Comprehensive checks before any modifications                           | 🔴 TODO | High     |
| Progress reporting    | Add progress bars for long operations                                   | 🔴 TODO | Medium   |
| Dry-run mode          | Preview operations without changes                                      | 🔴 TODO | Medium   |
| CI/CD setup           | GitHub Actions for multi-platform testing                               | 🔴 TODO | High     |

### Testing Requirements

| Test Type                  | Description                                                       | Status  | Priority |
|----------------------------|-------------------------------------------------------------------|---------|----------|
| CLI option tests           | Test --only, --ignore, --skip-existing, --sync-selection behavior | 🔴 TODO | Critical |
| Migration tests            | Verify deprecation warnings and flag compatibility                | 🔴 TODO | Critical |
| Sync selection tests       | Test removal logic with various selection combinations            | 🔴 TODO | Critical |
| Rollback integration tests | Test commit restoration in various failure scenarios              | 🔴 TODO | Critical |
| Atomic operation tests     | Verify all-or-nothing behavior with partial failures              | 🔴 TODO | High     |
| Validation edge cases      | Test pre-flight checks with dirty repos, conflicts, etc.          | 🔴 TODO | High     |
| Progress reporting tests   | Ensure accurate progress in normal and error cases                | 🔴 TODO | Medium   |
| Dry-run tests              | Verify no modifications occur in dry-run mode                     | 🔴 TODO | Medium   |
| CI matrix tests            | Test on Ubuntu, macOS, Windows with multiple Git versions         | 🔴 TODO | High     |
| Failure recovery tests     | Test recovery from network failures, auth errors, etc.            | 🔴 TODO | High     |

### Deliverables
- Redesigned CLI with clearer options and better defaults
- Full synchronization capabilities with --sync-selection
- Robust error recovery with full test coverage
- Better user feedback
- CI/CD pipeline with cross-platform testing
- 90%+ test coverage for error paths
- Migration guide for v0.3.x users

---

## Phase 3: Performance & UX (v0.5.0 - Q2 2024)

**Goal**: Improve performance and user experience

### Action Items

| Task                  | Description                                  | Status  | Priority |
|-----------------------|----------------------------------------------|---------|----------|
| Parallel processing   | Concurrent repository operations with limits | 🔴 TODO | High     |
| Configurable timeouts | Add timeout options for Git operations       | 🔴 TODO | Medium   |
| Verbose logging       | Optional detailed output for debugging       | 🔴 TODO | Low      |
| Checkpoint commits    | Create commits after successful batches      | 🔴 TODO | Low      |
| Memory optimization   | Improve handling of large repo lists         | 🔴 TODO | Low      |

### Testing Requirements

| Test Type                 | Description                                                  | Status  | Priority |
|---------------------------|--------------------------------------------------------------|---------|----------|
| Parallel processing tests | Test concurrent operations, race conditions, resource limits | 🔴 TODO | High     |
| Performance benchmarks    | Measure speedup with different repo counts and sizes         | 🔴 TODO | High     |
| Timeout tests             | Verify timeout behavior for slow/hung operations             | 🔴 TODO | Medium   |
| Memory usage tests        | Profile memory with 100+, 500+, 1000+ repositories           | 🔴 TODO | Medium   |
| Stress tests              | Test with very large repos, slow networks, limited resources | 🔴 TODO | Medium   |
| Platform-specific tests   | Windows path handling, macOS SSH agent, Linux permissions    | 🔴 TODO | High     |
| Logging level tests       | Verify verbose output contains expected information          | 🔴 TODO | Low      |

### Deliverables
- 5-10x performance improvement for large repo lists
- Better debugging capabilities
- Cross-platform test suite
- Performance regression tests

---

## Phase 4: Configuration & Flexibility (v0.6.0 - Q3 2025)

**Goal**: Add configuration options and advanced features

### Action Items

| Task                  | Description                                   | Status  | Priority |
|-----------------------|-----------------------------------------------|---------|----------|
| Configuration file    | Support .vcs2git.toml for persistent settings | 🔴 TODO | Medium   |
| Custom naming         | Allow different submodule names from paths    | 🔴 TODO | Low      |
| Template support      | Variables in .repos files                     | 🔴 TODO | Low      |
| Multiple auth methods | Add token and SSH key file support            | 🔴 TODO | Low      |
| Recursive processing  | Handle nested .repos files                    | 🔴 TODO | Low      |
| Flag aliases          | Support short flags (-o, -i, -s)              | 🔴 TODO | Low      |

### Testing Requirements

| Test Type                 | Description                                           | Status  | Priority |
|---------------------------|-------------------------------------------------------|---------|----------|
| Configuration tests       | Test loading, validation, and merging of config files | 🔴 TODO | Medium   |
| Auth method tests         | Test SSH keys, tokens, and fallback behavior          | 🔴 TODO | Medium   |
| Template expansion tests  | Test variable substitution in .repos files            | 🔴 TODO | Low      |
| Recursive operation tests | Test nested .repos with circular dependencies         | 🔴 TODO | Medium   |
| Config override tests     | Test CLI args override config file settings           | 🔴 TODO | Medium   |
| Flag alias tests          | Test short flag equivalence to long flags             | 🔴 TODO | Low      |

### Deliverables
- Flexible configuration with comprehensive tests
- Advanced repository management
- Enhanced authentication options
- Configuration migration guide

---

## Phase 5: Git-Specific Optimizations (v0.7.0 - Q4 2024)

**Goal**: Leverage Git-specific features for better performance

### Action Items

| Task                     | Description                            | Status  | Priority |
|--------------------------|----------------------------------------|---------|----------|
| Shallow clones           | Support --depth for submodules         | 🔴 TODO | Medium   |
| Sparse checkout          | Partial checkouts of large repos       | 🔴 TODO | Low      |
| Git worktree exploration | Research alternative to submodules     | 🔴 TODO | Low      |
| Batch operations         | Optimize Git commands for bulk updates | 🔴 TODO | Low      |

### Testing Requirements

| Test Type                  | Description                                        | Status  | Priority |
|----------------------------|----------------------------------------------------|---------|----------|
| Shallow clone tests        | Test --depth with various values and edge cases    | 🔴 TODO | Medium   |
| Sparse checkout tests      | Test partial checkouts with different patterns     | 🔴 TODO | Low      |
| Bandwidth tests            | Measure data transfer reduction with optimizations | 🔴 TODO | Medium   |
| Worktree compatibility     | Test worktree operations don't break submodules    | 🔴 TODO | Low      |
| History preservation tests | Verify shallow clones can be deepened later        | 🔴 TODO | Medium   |
| Large repo tests           | Test with repos >1GB, >100k files                  | 🔴 TODO | High     |

### Deliverables
- 50%+ bandwidth reduction for large repos
- Faster cloning of large repositories
- Alternative workflow documentation
- Optimization benchmark suite

---

## Phase 6: Ecosystem Integration (v1.0.0 - Q1 2025)

**Goal**: Achieve stable release with comprehensive features

### Action Items

| Task                   | Description                        | Status  | Priority |
|------------------------|------------------------------------|---------|----------|
| API stability          | Finalize CLI interface             | 🔴 TODO | High     |
| Documentation          | Comprehensive user guide           | 🔴 TODO | High     |
| Example repos          | Real-world usage examples          | 🔴 TODO | Medium   |
| IDE integration guides | VS Code, Vim setup guides          | 🔴 TODO | Low      |
| CI/CD templates        | GitHub Actions, GitLab CI examples | 🔴 TODO | Low      |
| Plugin architecture    | Extensibility research             | 🔴 TODO | Low      |

### Testing Requirements

| Test Type                | Description                                              | Status  | Priority |
|--------------------------|----------------------------------------------------------|---------|----------|
| API stability tests      | Test backward compatibility across versions              | 🔴 TODO | High     |
| Documentation tests      | Verify all examples in docs actually work                | 🔴 TODO | High     |
| Integration tests        | Test with popular ROS/Autoware repos                     | 🔴 TODO | High     |
| CLI compatibility tests  | Test with different shells (bash, zsh, fish, PowerShell) | 🔴 TODO | Medium   |
| Migration tests          | Test upgrading from older versions                       | 🔴 TODO | High     |
| End-to-end scenarios     | Test complete workflows from vcstool migration           | 🔴 TODO | High     |
| Community feedback tests | Beta testing with real users                             | 🔴 TODO | High     |

### Deliverables
- Stable 1.0 release with 95%+ test coverage
- Complete documentation with tested examples
- Community resources and migration guides
- Performance and compatibility test suites

---

## Known Technical Debt

### Immediate Issues (from known-issues.md)
1. **Logic bugs**: Disjoint check and error messages
2. **Missing validation**: Duplicate names, unclean states
3. **No tests**: Critical functions lack unit tests
4. **No rollback**: Need commit-based recovery mechanism

### Architecture Improvements
1. **Error handling**: Implement atomic operations with rollback
2. **Code organization**: Refactor main.rs into modules
3. **Type safety**: Stronger types for repository states
4. **State management**: Add SubmoduleStateTracker for recovery

### Documentation Needs
1. **Error reference**: Comprehensive troubleshooting guide
2. **Architecture docs**: Update after refactoring
3. **Migration guide**: From vcstool to vcs2git

---

## Non-Goals (Permanent)

### Explicitly Out of Scope
- ❌ **Other VCS Support**: Will never support Mercurial, SVN, Bazaar (Git-only by design)
- ❌ **Full VCS Tool**: Not a vcstool replacement
- ❌ **Repository Hosting**: No built-in Git hosting
- ❌ **Complex Workflows**: No git-flow support
- ❌ **GUI Interface**: CLI-only tool

---

## Success Metrics

### Phase 1-2 (Bug Fixes & Error Handling)
- Zero critical bugs in core functionality
- 90%+ test coverage for validation logic
- Clear error messages for all failure modes
- All tests passing on Linux, macOS, Windows

### Phase 3-4 (Performance & Configuration)
- 5-10x speedup for large repo lists (via parallelization)
- Support for 1000+ repository lists
- Configuration file adoption by users
- <5s response time for 100 repo operations

### Phase 5-6 (Optimization & Ecosystem)
- 50% bandwidth reduction with shallow clones
- Community-contributed integrations
- Stable API with no breaking changes
- 95%+ overall test coverage

## Testing Philosophy

### Test Categories
1. **Unit Tests**: Individual function behavior
2. **Integration Tests**: Multi-component interactions
3. **End-to-End Tests**: Complete user workflows
4. **Performance Tests**: Speed and resource usage
5. **Compatibility Tests**: Cross-platform behavior

### Test Coverage Goals
- Phase 1: 80%+ coverage for critical paths
- Phase 2: 90%+ coverage for error handling
- Phase 3-4: Include performance benchmarks
- Phase 5-6: 95%+ total coverage with regression tests

### Continuous Testing
- All PRs must pass existing tests
- New features require corresponding tests
- Performance tests run nightly
- Compatibility tests on release candidates

---

## Contributing Priorities

### High Impact (Immediate Need)
1. **Bug Fixes**: Fix disjoint check and error messages
2. **Testing**: Unit and integration tests
3. **Validation**: Clean state checks

### Medium Impact (Next Quarter)
1. **Performance**: Parallel processing implementation
2. **Documentation**: User guides and examples
3. **Platform Testing**: Windows/macOS compatibility

### Research Areas
1. **Git Worktrees**: Alternative to submodules
2. **Sparse Checkout**: Large repository optimization
3. **Security**: Audit of Git operations

---

## Release Schedule

| Version | Target Date | Focus Area                           | Status      |
|---------|-------------|--------------------------------------|-------------|
| v0.3.1  | Immediate   | Critical bug fixes                   | ✅ COMPLETED |
| v0.4.0  | Q1 2025     | CLI redesign, error handling & sync  | 🔴 Planning |
| v0.5.0  | Q2 2025     | Performance & UX                     | 🔴 Planning |
| v0.6.0  | Q3 2025     | Configuration & flexibility          | 🔴 Planning |
| v0.7.0  | Q4 2025     | Git optimizations                    | 🔴 Planning |
| v1.0.0  | Q1 2026     | Stable release                       | 🔴 Planning |

---

## Risk Mitigation

### Technical Risks
1. **Git2 library limitations**: May need to shell out for some operations
2. **Platform differences**: Windows path handling needs special attention
3. **Performance bottlenecks**: Network I/O may limit parallelization benefits

### Mitigation Strategies
1. **Incremental releases**: Small, focused updates
2. **Community feedback**: Early testing for each phase
3. **Escape hatches**: Alternative implementations for problematic features
