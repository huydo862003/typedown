#!/usr/bin/env bash
set -euo pipefail

read -rp "Staging version (e.g. 0.1.0-rc.1): " VERSION

if [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+-[a-zA-Z]+\.[0-9]+$ ]]; then
  echo "Error: '$VERSION' does not match expected format X.Y.Z-label.N"
  exit 1
fi

TAG="staging/v$VERSION"

git tag "$TAG"
git push origin "$TAG"

echo "Pushed tag $TAG"
