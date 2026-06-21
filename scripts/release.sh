#!/usr/bin/env bash
# Cut an immutable release for a commit, reusing the binaries CI already built.
# Usage: scripts/release.sh <commit>
#
# Tags v<version-at-commit>, fails if that tag exists, and attaches the CI
# artifacts for that exact commit. There is no separate release build.
set -euo pipefail

commit=${1:?usage: $0 <commit>}
root=$(git rev-parse --show-toplevel)
cd "$root"

sha=$(git rev-parse "$commit")
version=$(git show "$sha:Cargo.toml" | grep -m1 '^version = ' | cut -d'"' -f2)
tag="v$version"

# Immutable: refuse if the tag already exists locally or on the remote.
if git rev-parse -q --verify "refs/tags/$tag" >/dev/null \
  || git ls-remote --exit-code --tags origin "$tag" >/dev/null 2>&1; then
  echo "error: tag $tag already exists" >&2
  exit 1
fi

# Find the successful CI run for this commit and download its artifacts.
run_id=$(gh run list --commit "$sha" --workflow ci.yml \
  --json databaseId,conclusion \
  --jq '[.[] | select(.conclusion=="success")][0].databaseId')
if [ -z "$run_id" ] || [ "$run_id" = "null" ]; then
  echo "error: no successful ci.yml run found for $sha" >&2
  exit 1
fi

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
gh run download "$run_id" --dir "$tmp"

assets=()
for f in "$tmp"/*/ocaml-oxidizer-*; do
  [ -f "$f" ] || continue
  chmod +x "$f"
  assets+=("$f")
done
if [ "${#assets[@]}" -eq 0 ]; then
  echo "error: no release binaries among CI artifacts for $sha" >&2
  exit 1
fi

git tag "$tag" "$sha"
git push origin "$tag"

# gh release create fails if the release already exists -> immutable.
gh release create "$tag" --target "$sha" --title "$tag" \
  --notes "Release $tag (binaries reused from CI run $run_id)." "${assets[@]}"

echo "released $tag with ${#assets[@]} binaries"
