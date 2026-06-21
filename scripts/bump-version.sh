#!/usr/bin/env bash
# Bump the crate version and open a PR.
# Usage: scripts/bump-version.sh <major> <minor> <patch> <expected-current>
#
# The expected-current argument is a safety check: the bump aborts unless the
# crate is currently at that version.
set -euo pipefail

if [ "$#" -ne 4 ]; then
  echo "usage: $0 <major> <minor> <patch> <expected-current>" >&2
  exit 1
fi

new="$1.$2.$3"
expected="$4"

root=$(git rev-parse --show-toplevel)
cd "$root"

current=$(grep -m1 '^version = ' Cargo.toml | cut -d'"' -f2)
if [ "$current" != "$expected" ]; then
  echo "error: current version is $current, expected $expected" >&2
  exit 1
fi
if [ "$new" = "$current" ]; then
  echo "error: new version $new equals current version" >&2
  exit 1
fi

branch="bump-v$new"
git switch -c "$branch"

perl -0777 -pi -e 's/^version = "\Q'"$current"'\E"/version = "'"$new"'"/m' Cargo.toml
perl -0777 -pi -e 's/(name = "ocaml-oxidizer"\nversion = ")[^"]*(")/${1}'"$new"'${2}/' Cargo.lock

git add Cargo.toml Cargo.lock
git commit -m "chore: bump version to $new"
git push -u origin "$branch"

base=$(gh repo view --json defaultBranchRef --jq .defaultBranchRef.name)
gh pr create --base "$base" --head "$branch" \
  --title "Bump version to $new" \
  --body "Bumps version from $current to $new."
