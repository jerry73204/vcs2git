use anyhow::Result;
use git2::Repository;
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

/// Helper to create a test Git repository with initial commit
fn create_test_repo(path: &Path) -> Result<Repository> {
    let repo = Repository::init(path)?;

    // Create initial commit
    let sig = git2::Signature::now("Test User", "test@example.com")?;
    let tree_id = {
        let mut index = repo.index()?;
        index.write_tree()?
    };
    let tree = repo.find_tree(tree_id)?;

    repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])?;

    drop(tree); // Explicitly drop the tree before returning repo
    Ok(repo)
}

/// Helper to create a test .repos file
fn create_test_repos_file(path: &Path, content: &str) -> Result<()> {
    fs::write(path, content)?;
    Ok(())
}

#[test]
fn test_simple_add_submodule() -> Result<()> {
    // Create temporary directories
    let temp_dir = TempDir::new()?;
    let main_repo_path = temp_dir.path().join("main");
    let sub_repo_path = temp_dir.path().join("sub");

    // Create main repository
    fs::create_dir(&main_repo_path)?;
    let _main_repo = create_test_repo(&main_repo_path)?;

    // Create submodule repository
    fs::create_dir(&sub_repo_path)?;
    let _sub_repo = create_test_repo(&sub_repo_path)?;

    // Create .repos file
    let repos_content = format!(
        r#"repositories:
  test/sub:
    type: git
    url: file://{}
    version: main
"#,
        sub_repo_path.display()
    );

    let repos_file = main_repo_path.join("test.repos");
    create_test_repos_file(&repos_file, &repos_content)?;

    // Run vcs2git
    let output = Command::new(env!("CARGO_BIN_EXE_vcs2git"))
        .current_dir(&main_repo_path)
        .args(&[repos_file.to_str().unwrap(), "src"])
        .output()?;

    if !output.status.success() {
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        panic!("vcs2git failed");
    }

    // Verify submodule was added
    let gitmodules_path = main_repo_path.join(".gitmodules");
    assert!(gitmodules_path.exists(), ".gitmodules should exist");

    let submodule_path = main_repo_path.join("src/test/sub");
    assert!(submodule_path.exists(), "Submodule directory should exist");

    Ok(())
}

#[test]
fn test_validation_dirty_repo() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("repo");

    // Create repository
    fs::create_dir(&repo_path)?;
    let repo = create_test_repo(&repo_path)?;

    // Create a file and stage it
    let test_file = repo_path.join("test.txt");
    fs::write(&test_file, "test content")?;

    let mut index = repo.index()?;
    index.add_path(Path::new("test.txt"))?;
    index.write()?;

    // Create empty .repos file
    let repos_content = "repositories: {}";
    let repos_file = repo_path.join("test.repos");
    create_test_repos_file(&repos_file, &repos_content)?;

    // Run vcs2git - should fail due to staged changes
    let output = Command::new(env!("CARGO_BIN_EXE_vcs2git"))
        .current_dir(&repo_path)
        .args(&[repos_file.to_str().unwrap(), "src"])
        .output()?;

    assert!(
        !output.status.success(),
        "vcs2git should fail with staged changes"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("staged changes"),
        "Error message should mention staged changes"
    );

    Ok(())
}

#[test]
fn test_rollback_on_failure() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let main_repo_path = temp_dir.path().join("main");

    // Create main repository
    fs::create_dir(&main_repo_path)?;
    let _main_repo = create_test_repo(&main_repo_path)?;

    // Create .repos file with non-existent repository
    let repos_content = r#"repositories:
  test/nonexistent:
    type: git
    url: file:///nonexistent/repo
    version: main
"#;

    let repos_file = main_repo_path.join("test.repos");
    create_test_repos_file(&repos_file, &repos_content)?;

    // Run vcs2git - should fail
    let output = Command::new(env!("CARGO_BIN_EXE_vcs2git"))
        .current_dir(&main_repo_path)
        .args(&[repos_file.to_str().unwrap(), "src"])
        .output()?;

    if output.status.success() {
        panic!("vcs2git should fail with non-existent repo");
    }

    // Debug output
    eprintln!("Exit status: {}", output.status);
    eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));

    // Check directory contents
    eprintln!("Directory contents:");
    for entry in fs::read_dir(&main_repo_path)? {
        let entry = entry?;
        eprintln!("  {}", entry.path().display());
    }

    // Check git status
    let git_status = Command::new("git")
        .current_dir(&main_repo_path)
        .args(&["status", "--porcelain"])
        .output()?;
    eprintln!(
        "Git status:\n{}",
        String::from_utf8_lossy(&git_status.stdout)
    );

    // Verify rollback cleaned up as much as possible
    let gitmodules_path = main_repo_path.join(".gitmodules");
    if gitmodules_path.exists() {
        // If .gitmodules exists, it should be empty or only have the failed entry
        let contents = fs::read_to_string(&gitmodules_path)?;
        eprintln!(".gitmodules contents:\n{}", contents);

        // Check that no actual submodule directories were created
        let submodule_path = main_repo_path.join("src/test/nonexistent");
        assert!(
            !submodule_path.join(".git").exists(),
            "Submodule .git directory should not exist after rollback"
        );
    }

    // The key check: ensure no submodule was actually initialized
    let repo = Repository::open(&main_repo_path)?;
    let submodules = repo.submodules()?;
    assert_eq!(
        submodules.len(),
        0,
        "No submodules should be initialized after rollback"
    );

    Ok(())
}

#[test]
fn test_unsupported_repository_type() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path().join("repo");

    // Create repository
    fs::create_dir(&repo_path)?;
    let _repo = create_test_repo(&repo_path)?;

    // Create .repos file with unsupported type
    let repos_content = r#"repositories:
  test/hg-repo:
    type: hg
    url: https://example.com/repo
    version: default
"#;

    let repos_file = repo_path.join("test.repos");
    create_test_repos_file(&repos_file, &repos_content)?;

    // Run vcs2git - should fail
    let output = Command::new(env!("CARGO_BIN_EXE_vcs2git"))
        .current_dir(&repo_path)
        .args(&[repos_file.to_str().unwrap(), "src"])
        .output()?;

    assert!(
        !output.status.success(),
        "vcs2git should fail with unsupported type"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("'hg' is not supported"),
        "Error message should mention unsupported type"
    );

    Ok(())
}
