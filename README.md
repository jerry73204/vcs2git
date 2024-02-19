# **vcs2git**: Convert VCS repos to Git Submodules

## Installation

Install the binary using Cargo.

```bash
cargo install --git https://github.com/jerry73204/vcs2git.git
```

## Usage

This program reads a YAML .repos file and adds listed repos as
submodules in the current Git repository.


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


# License

This software is distributed under MIT license. Please see the
[license file](LICENSE.txt).
