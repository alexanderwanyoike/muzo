#!/usr/bin/env bash
set -euo pipefail

remote=$1
ref="refs/tags/$2"
expected_tag=$3
expected_commit=$4

references=$(git ls-remote "$remote" "$ref" "${ref}^{}")
if ! grep -Fxq -- "$expected_tag"$'\t'"$ref" <<< "$references" ||
   ! grep -Fxq -- "$expected_commit"$'\t'"${ref}^{}" <<< "$references"; then
  echo "Release tag $2 no longer points to the verified source." >&2
  exit 1
fi
