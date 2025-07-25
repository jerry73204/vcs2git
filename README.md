# **vcs2git**: Convert VCS repos to Git Submodules

## Installation

Install the binary using Cargo.

```bash
cargo install vcs2git
```

## Usage

This program reads a YAML .repos file and adds listed repos as
submodules in the current Git repository.

### Basic Usage

Let's get started with Autoware's
[autoware.repos](https://github.com/autowarefoundation/autoware/blob/af0fbe322ba075ee4b4f0f87789c43b54800a234/autoware.repos)
for example.

```bash
# Enter into the root of your Git repo
cd my_repo

# Add listed repos in autoware.repos as submodules under src directory.
mkdir src
vcs2git autoware.repos src

# Save added submodules
git commit
```

### Command Line Options

```
vcs2git [OPTIONS] <REPO_FILE> <PREFIX>

Arguments:
  <REPO_FILE>  The YAML file of a repository list
  <PREFIX>     The directory to add submodules

Options:
  --only <REPO>...           Process only these repositories
  --ignore <REPO>...         Process all repositories except these
  --skip-existing            Don't update existing submodules (by default, existing submodules are updated)
  --sync-selection           Remove submodules that are not in the current selection
  --no-checkout              Do not checkout the files in each submodule
  --dry-run                  Preview what would be done without making changes
  -h, --help                 Print help
```

### Advanced Examples

#### Process Only Specific Repositories

```bash
# Only add specific repositories
vcs2git autoware.repos src --only universe/autoware.universe universe/external

# Process all except specific repositories
vcs2git autoware.repos src --ignore universe/external
```

#### Synchronize with Repository File

```bash
# Remove submodules not listed in the repos file
vcs2git autoware.repos src --sync-selection

# Keep only specific repositories and remove all others
vcs2git autoware.repos src --only core/autoware --sync-selection
```

#### Skip Updating Existing Submodules

```bash
# Add new submodules but don't update existing ones
vcs2git autoware.repos src --skip-existing
```

#### Preview Changes (Dry Run)

```bash
# See what would be done without making changes
vcs2git autoware.repos src --dry-run

# Preview sync operation
vcs2git autoware.repos src --sync-selection --dry-run
```

# License

This software is distributed under MIT license. Please see the
[license file](LICENSE.txt).
