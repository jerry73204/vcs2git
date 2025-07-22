use color_eyre::Result;
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

    drop(tree);
    Ok(repo)
}

/// Helper to create a test repo with content
fn create_test_repo_with_content(
    path: &Path,
    file_name: &str,
    content: &str,
) -> Result<Repository> {
    let repo = Repository::init(path)?;

    // Create a file
    fs::write(path.join(file_name), content)?;

    // Add and commit
    let sig = git2::Signature::now("Test User", "test@example.com")?;
    let mut index = repo.index()?;
    index.add_path(Path::new(file_name))?;
    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;

    repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])?;

    drop(tree);
    Ok(repo)
}

#[test]
fn test_rollback_on_partial_failure() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let main_repo_path = temp_dir.path().join("main");
    fs::create_dir(&main_repo_path)?;
    let main_repo = create_test_repo(&main_repo_path)?;

    // Create two valid repos and one that will fail
    let repo1_path = temp_dir.path().join("repo1");
    let repo2_path = temp_dir.path().join("repo2");
    fs::create_dir(&repo1_path)?;
    fs::create_dir(&repo2_path)?;
    create_test_repo(&repo1_path)?;
    create_test_repo(&repo2_path)?;

    // Create .repos file with two valid and one invalid repo
    let repos_content = format!(
        r#"repositories:
  test/repo1:
    type: git
    url: file://{}
    version: main
  test/invalid:
    type: git
    url: file:///nonexistent/repo
    version: main
  test/repo2:
    type: git
    url: file://{}
    version: main
"#,
        repo1_path.display(),
        repo2_path.display()
    );

    let repos_file = main_repo_path.join("test.repos");
    fs::write(&repos_file, &repos_content)?;

    // Get initial state
    let initial_submodules = main_repo.submodules()?.len();

    // Run vcs2git - should fail on invalid repo
    let output = Command::new(env!("CARGO_BIN_EXE_vcs2git"))
        .current_dir(&main_repo_path)
        .args([repos_file.to_str().unwrap(), "src"])
        .output()?;

    assert!(!output.status.success());

    // Verify rollback: no submodules should be added
    let final_repo = Repository::open(&main_repo_path)?;
    let final_submodules = final_repo.submodules()?.len();
    assert_eq!(
        initial_submodules, final_submodules,
        "Rollback should restore original state"
    );

    // Verify no partial directories remain
    assert!(!main_repo_path.join("src/test/repo1").exists());
    assert!(!main_repo_path.join("src/test/invalid").exists());
    assert!(!main_repo_path.join("src/test/repo2").exists());

    Ok(())
}

#[test]
fn test_rollback_restores_existing_submodule_commits() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let main_repo_path = temp_dir.path().join("main");
    fs::create_dir(&main_repo_path)?;
    let main_repo = create_test_repo(&main_repo_path)?;

    // Create a submodule repo with two commits
    let sub_repo_path = temp_dir.path().join("sub");
    fs::create_dir(&sub_repo_path)?;
    let sub_repo = create_test_repo_with_content(&sub_repo_path, "file1.txt", "version 1")?;

    // Create second commit
    fs::write(sub_repo_path.join("file2.txt"), "version 2")?;
    let sig = git2::Signature::now("Test User", "test@example.com")?;
    let mut index = sub_repo.index()?;
    index.add_path(Path::new("file2.txt"))?;
    let tree_id = index.write_tree()?;
    let tree = sub_repo.find_tree(tree_id)?;
    let parent = sub_repo.head()?.peel_to_commit()?;
    let second_commit =
        sub_repo.commit(Some("HEAD"), &sig, &sig, "Second commit", &tree, &[&parent])?;

    // Get first commit ID
    let first_commit = parent.id();

    // Add submodule at first commit
    let repos_content_v1 = format!(
        r#"repositories:
  test/sub:
    type: git
    url: file://{}
    version: {}
"#,
        sub_repo_path.display(),
        first_commit
    );

    let repos_file = main_repo_path.join("test.repos");
    fs::write(&repos_file, &repos_content_v1)?;

    // Add submodule
    let output = Command::new(env!("CARGO_BIN_EXE_vcs2git"))
        .current_dir(&main_repo_path)
        .args([repos_file.to_str().unwrap(), "src"])
        .output()?;

    assert!(output.status.success());

    // Commit the addition
    let sig = git2::Signature::now("Test User", "test@example.com")?;
    let mut index = main_repo.index()?;
    index.write_tree()?;
    let tree_id = index.write_tree()?;
    let tree = main_repo.find_tree(tree_id)?;
    let parent = main_repo.head()?.peel_to_commit()?;
    main_repo.commit(Some("HEAD"), &sig, &sig, "Add submodule", &tree, &[&parent])?;

    // Verify submodule is at first commit
    let submodule = main_repo.find_submodule("src/test/sub")?;
    assert_eq!(submodule.workdir_id().unwrap(), first_commit);

    // Now try to update to second commit but with an invalid second repo
    let repos_content_v2 = format!(
        r#"repositories:
  test/sub:
    type: git
    url: file://{}
    version: {}
  test/invalid:
    type: git
    url: file:///nonexistent/repo
    version: main
"#,
        sub_repo_path.display(),
        second_commit
    );

    fs::write(&repos_file, &repos_content_v2)?;

    // Run vcs2git - should fail on invalid repo
    let output = Command::new(env!("CARGO_BIN_EXE_vcs2git"))
        .current_dir(&main_repo_path)
        .args([repos_file.to_str().unwrap(), "src"])
        .output()?;

    assert!(!output.status.success());

    // Verify submodule was restored to first commit
    let main_repo = Repository::open(&main_repo_path)?;
    let submodule = main_repo.find_submodule("src/test/sub")?;
    assert_eq!(
        submodule.workdir_id().unwrap(),
        first_commit,
        "Submodule should be restored to original commit"
    );

    Ok(())
}

#[test]
fn test_rollback_with_sync_selection() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let main_repo_path = temp_dir.path().join("main");
    fs::create_dir(&main_repo_path)?;
    let main_repo = create_test_repo(&main_repo_path)?;

    // Create repos
    let repo1_path = temp_dir.path().join("repo1");
    let repo2_path = temp_dir.path().join("repo2");
    fs::create_dir(&repo1_path)?;
    fs::create_dir(&repo2_path)?;
    create_test_repo(&repo1_path)?;
    create_test_repo(&repo2_path)?;

    // First add both repos
    let repos_content = format!(
        r#"repositories:
  test/repo1:
    type: git
    url: file://{}
    version: main
  test/repo2:
    type: git
    url: file://{}
    version: main
"#,
        repo1_path.display(),
        repo2_path.display()
    );

    let repos_file = main_repo_path.join("test.repos");
    fs::write(&repos_file, &repos_content)?;

    // Add both repos
    let output = Command::new(env!("CARGO_BIN_EXE_vcs2git"))
        .current_dir(&main_repo_path)
        .args([repos_file.to_str().unwrap(), "src"])
        .output()?;

    assert!(output.status.success());

    // Commit
    let sig = git2::Signature::now("Test User", "test@example.com")?;
    let mut index = main_repo.index()?;
    let tree_id = index.write_tree()?;
    let tree = main_repo.find_tree(tree_id)?;
    let parent = main_repo.head()?.peel_to_commit()?;
    main_repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        "Add submodules",
        &tree,
        &[&parent],
    )?;

    // Now create repos file with invalid repo and try to sync
    let repos_content_invalid = format!(
        r#"repositories:
  test/repo1:
    type: git
    url: file://{}
    version: main
  test/invalid:
    type: git
    url: file:///nonexistent/repo
    version: main
"#,
        repo1_path.display()
    );

    fs::write(&repos_file, &repos_content_invalid)?;

    // Try to sync - should fail but restore original state
    let output = Command::new(env!("CARGO_BIN_EXE_vcs2git"))
        .current_dir(&main_repo_path)
        .args([repos_file.to_str().unwrap(), "src", "--sync-selection"])
        .output()?;

    assert!(!output.status.success());

    // Verify both original repos still exist (rollback should have restored them)
    assert!(main_repo_path.join("src/test/repo1").exists());
    assert!(main_repo_path.join("src/test/repo2").exists());

    Ok(())
}

#[test]
fn test_dry_run_rollback_not_triggered() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let main_repo_path = temp_dir.path().join("main");
    fs::create_dir(&main_repo_path)?;
    let _main_repo = create_test_repo(&main_repo_path)?;

    // Create repos file with invalid repo
    let repos_content = r#"repositories:
  test/invalid:
    type: git
    url: file:///nonexistent/repo
    version: main
"#;

    let repos_file = main_repo_path.join("test.repos");
    fs::write(&repos_file, repos_content)?;

    // Run with --dry-run - should report what would happen but not fail
    let output = Command::new(env!("CARGO_BIN_EXE_vcs2git"))
        .current_dir(&main_repo_path)
        .args([repos_file.to_str().unwrap(), "src", "--dry-run"])
        .output()?;

    // In dry-run mode, it should succeed since no actual operations are performed
    assert!(output.status.success());

    // Don't check for specific output since progress bar output isn't captured
    // The behavior test below verifies dry-run worked correctly

    // Verify no changes were made
    assert!(!main_repo_path.join(".gitmodules").exists());

    Ok(())
}
