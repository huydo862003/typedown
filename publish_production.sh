#!/usr/bin/env bash
set -euo pipefail

read -rp "Production version (e.g. 0.1.0): " VERSION

if [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "Error: '$VERSION' does not match expected format X.Y.Z"
  exit 1
fi

TAG="v$VERSION"

git tag "$TAG"
git push origin "$TAG"

echo "Pushed tag $TAG"
