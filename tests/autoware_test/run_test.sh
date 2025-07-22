#!/usr/bin/env bash
set -e

script_dir=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
cd "$script_dir"

manifest_path="${script_dir}/../../Cargo.toml"
cargo build --release --manifest-path "${manifest_path}"
prog="${script_dir}/../../target/release/vcs2git"

# Initialize the repository
rm -rf autoware_ws
mkdir autoware_ws
cd autoware_ws
git init

# Pull version 2025.02 submodules
"$prog" ../2025.02.repos src
git commit -m '2025.02'

# Pull version 0.45.0 submodules
"$prog" ../0.45.0.repos src --sync-selection
git commit -m '0.45.0'
