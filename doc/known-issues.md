# Known Issues and Limitations

## Current Limitations

### 1. Authentication Scope
**Issue**: Only SSH agent authentication is supported
**Impact**: Users without SSH agent setup cannot access private repositories
**Workaround**: Set up SSH agent with appropriate keys before running vcs2git
**Tracking**: No issue tracker yet - community feedback needed

### 2. Sequential Processing
**Issue**: Repositories are processed one at a time
**Impact**: Long execution time for large `.repos` files with many repositories
**Workaround**: None available - inherent to current architecture
**Planned Fix**: Parallel processing in v0.4.0

### 3. Limited VCS Support
**Issue**: Only Git repositories are supported (`type: git`)
**Impact**: Cannot convert repositories using Mercurial, SVN, or Bazaar
**Workaround**: Manually convert non-Git repositories to Git before using tool
**Status**: By design - may be addressed in future versions

### 4. No Rollback Mechanism
**Issue**: Failed operations may leave repository in partial state
**Impact**: Users must manually clean up failed submodule additions
**Workaround**: Use `git submodule deinit` and `git rm` to clean up manually
**Planned Fix**: Better error recovery in v0.4.0

## Known Bugs

### Error Message Issues

#### 1. Misleading Repository Type Error
**Bug**: Error message says "Repository type 'X' is supported" when it should say "not supported"
**Location**: `src/main.rs:120`
**Severity**: Low (cosmetic)
**Workaround**: Ignore the confusing message text
**Fix Required**: Change error message to "Repository type '{ty}' is not supported"

#### 2. Logic Error in Disjoint Check
**Bug**: `check_disjoint` function has inverted logic
**Location**: `src/main.rs:225`
**Severity**: High (functional)
**Impact**: May allow repositories to be both selected and skipped
**Workaround**: Avoid using `--select` and `--skip` for the same repositories
**Fix Required**: Change `if lset.is_disjoint(rset)` to `if !lset.is_disjoint(rset)`

## Platform-Specific Issues

### Windows
**Issue**: Path handling may not work correctly with Windows-style paths
**Impact**: Potential issues with backslash vs forward slash in submodule paths
**Status**: Needs testing - no confirmed reports
**Workaround**: Use forward slashes in `.repos` files

### macOS
**Issue**: SSH agent integration may differ from Linux
**Impact**: Authentication failures on some macOS configurations
**Status**: Needs investigation - limited testing
**Workaround**: Ensure SSH agent is properly configured

## Performance Issues

### Large Repository Lists
**Issue**: Memory usage scales linearly with number of repositories
**Impact**: Potential memory issues with very large `.repos` files (1000+ repos)
**Severity**: Low (uncommon use case)
**Workaround**: Split large `.repos` files into smaller chunks

### Network Timeouts
**Issue**: No configurable timeouts for Git operations
**Impact**: Tool may hang on slow or unresponsive repositories
**Workaround**: Use `Ctrl+C` to interrupt and manually skip problematic repositories
**Planned Fix**: Configurable timeouts in v0.5.0

## Edge Cases

### Conflicting Submodule Names
**Issue**: Git submodule names must be unique, but tool doesn't validate this
**Impact**: Git errors when adding submodules with duplicate names
**Workaround**: Ensure unique paths in `.repos` file
**Fix Required**: Pre-validation of submodule name uniqueness

### Empty Repository Directories
**Issue**: Tool doesn't handle repositories that exist but are empty
**Impact**: Checkout operations may fail with unclear error messages
**Workaround**: Remove empty directories before running tool

### Branch vs Tag Ambiguity
**Issue**: When branch and tag have the same name, behavior is undefined
**Impact**: May checkout different revision than expected
**Workaround**: Use commit hashes for unambiguous versioning

## Security Considerations

### Repository Trust
**Issue**: Tool trusts all repository URLs without validation
**Impact**: Potential for malicious repository URLs to be processed
**Mitigation**: Tool performs no code execution, only Git operations
**Recommendation**: Validate `.repos` files from untrusted sources

### SSH Key Exposure
**Issue**: SSH agent integration may expose keys to malicious repositories
**Impact**: Private keys could be used for unauthorized access
**Mitigation**: Standard SSH agent security practices apply
**Recommendation**: Use dedicated SSH keys for automation

## Documentation Gaps

### Missing Examples
**Issue**: Limited examples for complex use cases
**Impact**: Users may not understand advanced features
**Status**: Documentation improvements planned

### Error Reference
**Issue**: No comprehensive error code reference
**Impact**: Difficult to troubleshoot specific error conditions
**Planned**: Error code documentation in v0.4.0

## Testing Limitations

### CI/CD Coverage
**Issue**: Limited automated testing on different platforms
**Impact**: Platform-specific bugs may not be caught
**Status**: GitHub Actions needed for comprehensive testing

### Integration Testing
**Issue**: No integration tests with real repositories
**Impact**: Real-world edge cases may not be covered
**Planned**: Integration test suite in v0.4.0

## Reporting Issues

To report new issues or confirm existing ones:

1. **Check Existing Issues**: Review this document and GitHub issues
2. **Provide Details**: Include `.repos` file, command line, error output
3. **System Information**: OS, Rust version, Git version
4. **Reproducible Case**: Minimal example that demonstrates the issue

**Note**: Issue tracker will be established based on community adoption and feedback.
