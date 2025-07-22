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

    drop(tree);
    Ok(repo)
}

/// Helper to create test submodule repositories
fn setup_test_repos(temp_dir: &TempDir) -> Result<(String, String, String)> {
    let repo1_path = temp_dir.path().join("repo1");
    let repo2_path = temp_dir.path().join("repo2");
    let repo3_path = temp_dir.path().join("repo3");

    fs::create_dir(&repo1_path)?;
    fs::create_dir(&repo2_path)?;
    fs::create_dir(&repo3_path)?;

    create_test_repo(&repo1_path)?;
    create_test_repo(&repo2_path)?;
    create_test_repo(&repo3_path)?;

    Ok((
        repo1_path.to_string_lossy().to_string(),
        repo2_path.to_string_lossy().to_string(),
        repo3_path.to_string_lossy().to_string(),
    ))
}

#[test]
fn test_only_flag() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let main_repo_path = temp_dir.path().join("main");
    fs::create_dir(&main_repo_path)?;
    let _main_repo = create_test_repo(&main_repo_path)?;

    let (repo1, repo2, repo3) = setup_test_repos(&temp_dir)?;

    // Create .repos file with three repositories
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
  test/repo3:
    type: git
    url: file://{}
    version: main
"#,
        repo1, repo2, repo3
    );

    let repos_file = main_repo_path.join("test.repos");
    fs::write(&repos_file, &repos_content)?;

    // Run vcs2git with --only flag
    let output = Command::new(env!("CARGO_BIN_EXE_vcs2git"))
        .current_dir(&main_repo_path)
        .args(&[
            repos_file.to_str().unwrap(),
            "src",
            "--only",
            "test/repo1",
            "--only",
            "test/repo2",
        ])
        .output()?;

    if !output.status.success() {
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        panic!("vcs2git failed");
    }

    // Verify only repo1 and repo2 were added
    assert!(main_repo_path.join("src/test/repo1").exists());
    assert!(main_repo_path.join("src/test/repo2").exists());
    assert!(!main_repo_path.join("src/test/repo3").exists());

    Ok(())
}

#[test]
fn test_ignore_flag() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let main_repo_path = temp_dir.path().join("main");
    fs::create_dir(&main_repo_path)?;
    let _main_repo = create_test_repo(&main_repo_path)?;

    let (repo1, repo2, repo3) = setup_test_repos(&temp_dir)?;

    // Create .repos file with three repositories
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
  test/repo3:
    type: git
    url: file://{}
    version: main
"#,
        repo1, repo2, repo3
    );

    let repos_file = main_repo_path.join("test.repos");
    fs::write(&repos_file, &repos_content)?;

    // Run vcs2git with --ignore flag
    let output = Command::new(env!("CARGO_BIN_EXE_vcs2git"))
        .current_dir(&main_repo_path)
        .args(&[
            repos_file.to_str().unwrap(),
            "src",
            "--ignore",
            "test/repo2",
        ])
        .output()?;

    if !output.status.success() {
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        panic!("vcs2git failed");
    }

    // Verify repo1 and repo3 were added, but not repo2
    assert!(main_repo_path.join("src/test/repo1").exists());
    assert!(!main_repo_path.join("src/test/repo2").exists());
    assert!(main_repo_path.join("src/test/repo3").exists());

    Ok(())
}

// Test removed - --select flag was removed in favor of --only

#[test]
fn test_skip_existing_flag() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let main_repo_path = temp_dir.path().join("main");
    fs::create_dir(&main_repo_path)?;
    let main_repo = create_test_repo(&main_repo_path)?;

    let (repo1, repo2, _) = setup_test_repos(&temp_dir)?;

    // Create .repos file
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
        repo1, repo2
    );

    let repos_file = main_repo_path.join("test.repos");
    fs::write(&repos_file, &repos_content)?;

    // First, add repo1 only
    let output = Command::new(env!("CARGO_BIN_EXE_vcs2git"))
        .current_dir(&main_repo_path)
        .args(&[repos_file.to_str().unwrap(), "src", "--only", "test/repo1"])
        .output()?;

    assert!(output.status.success());

    // Commit the changes to avoid "staged changes" error
    let sig = git2::Signature::now("Test User", "test@example.com")?;
    let mut index = main_repo.index()?;

    // Update the index to match HEAD first (removing any stale entries)
    index.read(true)?;

    // Add .gitmodules file to the index
    index.add_path(Path::new(".gitmodules"))?;
    index.write()?;

    let tree_id = index.write_tree()?;
    let tree = main_repo.find_tree(tree_id)?;
    let parent = main_repo.head()?.peel_to_commit()?;
    main_repo.commit(Some("HEAD"), &sig, &sig, "Add repo1", &tree, &[&parent])?;

    // Now run with --skip-existing, it should add repo2 but skip repo1
    let output = Command::new(env!("CARGO_BIN_EXE_vcs2git"))
        .current_dir(&main_repo_path)
        .args(&[repos_file.to_str().unwrap(), "src", "--skip-existing"])
        .output()?;

    if !output.status.success() {
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        panic!("vcs2git failed");
    }

    // Don't check for specific output since progress bar output isn't captured
    // Just verify the behavior is correct

    // Both should exist now
    assert!(main_repo_path.join("src/test/repo1").exists());
    assert!(main_repo_path.join("src/test/repo2").exists());

    Ok(())
}

#[test]
fn test_dry_run_mode() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let main_repo_path = temp_dir.path().join("main");
    fs::create_dir(&main_repo_path)?;
    let _main_repo = create_test_repo(&main_repo_path)?;

    let (repo1, _, _) = setup_test_repos(&temp_dir)?;

    let repos_content = format!(
        r#"repositories:
  test/repo1:
    type: git
    url: file://{}
    version: main
"#,
        repo1
    );

    let repos_file = main_repo_path.join("test.repos");
    fs::write(&repos_file, &repos_content)?;

    // Run with --dry-run
    let output = Command::new(env!("CARGO_BIN_EXE_vcs2git"))
        .current_dir(&main_repo_path)
        .args(&[repos_file.to_str().unwrap(), "src", "--dry-run"])
        .output()?;

    assert!(output.status.success());
    // Don't check for specific output since progress bar output isn't captured
    // The behavior test below verifies dry-run worked correctly

    // Verify no changes were made
    assert!(!main_repo_path.join("src/test/repo1").exists());
    assert!(!main_repo_path.join(".gitmodules").exists());

    Ok(())
}

#[test]
fn test_sync_selection_remove_unlisted() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let main_repo_path = temp_dir.path().join("main");
    fs::create_dir(&main_repo_path)?;
    let main_repo = create_test_repo(&main_repo_path)?;

    let (repo1, repo2, repo3) = setup_test_repos(&temp_dir)?;

    // First, add all three repos manually
    let repos_content_all = format!(
        r#"repositories:
  test/repo1:
    type: git
    url: file://{}
    version: main
  test/repo2:
    type: git
    url: file://{}
    version: main
  test/repo3:
    type: git
    url: file://{}
    version: main
"#,
        repo1, repo2, repo3
    );

    let repos_file = main_repo_path.join("test.repos");
    fs::write(&repos_file, &repos_content_all)?;

    // Add all repos first
    let output = Command::new(env!("CARGO_BIN_EXE_vcs2git"))
        .current_dir(&main_repo_path)
        .args(&[repos_file.to_str().unwrap(), "src"])
        .output()?;

    assert!(output.status.success());

    // Commit the changes - need to add all files including .gitmodules and submodule entries
    let sig = git2::Signature::now("Test User", "test@example.com")?;
    let mut index = main_repo.index()?;
    // Add .gitmodules file and all submodule entries to the index
    index.add_path(Path::new(".gitmodules"))?;
    index.add_path(Path::new("src/test/repo1"))?;
    index.add_path(Path::new("src/test/repo2"))?;
    index.add_path(Path::new("src/test/repo3"))?;
    index.write()?;
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

    // Now create a new repos file with only repo1 and repo2
    let repos_content_partial = format!(
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
        repo1, repo2
    );

    fs::write(&repos_file, &repos_content_partial)?;

    // Run with --sync-selection to remove repo3
    let output = Command::new(env!("CARGO_BIN_EXE_vcs2git"))
        .current_dir(&main_repo_path)
        .args(&[repos_file.to_str().unwrap(), "src", "--sync-selection"])
        .output()?;

    if !output.status.success() {
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        panic!("vcs2git failed");
    }

    // Verify repo1 and repo2 still exist, but repo3 was removed
    assert!(main_repo_path.join("src/test/repo1").exists());
    assert!(main_repo_path.join("src/test/repo2").exists());
    assert!(!main_repo_path.join("src/test/repo3").exists());

    Ok(())
}

#[test]
fn test_sync_selection_with_only() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let main_repo_path = temp_dir.path().join("main");
    fs::create_dir(&main_repo_path)?;
    let main_repo = create_test_repo(&main_repo_path)?;

    let (repo1, repo2, repo3) = setup_test_repos(&temp_dir)?;

    // Add all three repos first
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
  test/repo3:
    type: git
    url: file://{}
    version: main
"#,
        repo1, repo2, repo3
    );

    let repos_file = main_repo_path.join("test.repos");
    fs::write(&repos_file, &repos_content)?;

    // Add all repos
    let output = Command::new(env!("CARGO_BIN_EXE_vcs2git"))
        .current_dir(&main_repo_path)
        .args(&[repos_file.to_str().unwrap(), "src"])
        .output()?;

    assert!(output.status.success());

    // Commit the changes - need to add all files including .gitmodules and submodule entries
    let sig = git2::Signature::now("Test User", "test@example.com")?;
    let mut index = main_repo.index()?;
    // Add .gitmodules file and all submodule entries to the index
    index.add_path(Path::new(".gitmodules"))?;
    index.add_path(Path::new("src/test/repo1"))?;
    index.add_path(Path::new("src/test/repo2"))?;
    index.add_path(Path::new("src/test/repo3"))?;
    index.write()?;
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

    // Run with --only and --sync-selection to keep only repo1
    let output = Command::new(env!("CARGO_BIN_EXE_vcs2git"))
        .current_dir(&main_repo_path)
        .args(&[
            repos_file.to_str().unwrap(),
            "src",
            "--only",
            "test/repo1",
            "--sync-selection",
        ])
        .output()?;

    if !output.status.success() {
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        panic!("vcs2git failed");
    }

    // Verify only repo1 exists
    assert!(main_repo_path.join("src/test/repo1").exists());
    assert!(!main_repo_path.join("src/test/repo2").exists());
    assert!(!main_repo_path.join("src/test/repo3").exists());

    Ok(())
}

#[test]
fn test_basic_operation() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let main_repo_path = temp_dir.path().join("main");
    fs::create_dir(&main_repo_path)?;
    let _main_repo = create_test_repo(&main_repo_path)?;

    let (repo1, _, _) = setup_test_repos(&temp_dir)?;

    let repos_content = format!(
        r#"repositories:
  test/repo1:
    type: git
    url: file://{}
    version: main
"#,
        repo1
    );

    let repos_file = main_repo_path.join("test.repos");
    fs::write(&repos_file, &repos_content)?;

    // Run normally (progress is now always shown when there are operations)
    let output = Command::new(env!("CARGO_BIN_EXE_vcs2git"))
        .current_dir(&main_repo_path)
        .args(&[repos_file.to_str().unwrap(), "src"])
        .output()?;

    assert!(output.status.success());
    // Just verify the basic command succeeds

    Ok(())
}
