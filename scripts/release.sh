#!/bin/bash

set -e

VERSION=$1

if [ -z "$VERSION" ]; then
  echo "Usage: ./scripts/release.sh vX.Y.Z"
  echo "Example: ./scripts/release.sh v1.0.0"
  exit 1
fi

echo "=========================================="
echo "  SQLRustGo Release Script"
echo "=========================================="
echo ""
echo "Version: $VERSION"
echo ""

echo "🔍 Checking current branch..."
BRANCH=$(git branch --show-current)
echo "Current branch: $BRANCH"
echo ""

echo "🔍 Checking for uncommitted changes..."
if [ -n "$(git status --porcelain)" ]; then
  echo "❌ Error: You have uncommitted changes"
  echo "Please commit or stash your changes first"
  git status
  exit 1
fi
echo "✅ No uncommitted changes"
echo ""

echo "🔍 Pulling latest changes..."
git pull origin "$BRANCH"
echo ""

echo "🏷️  Creating tag $VERSION..."
git tag -a "$VERSION" -m "Release $VERSION"
echo ""

echo "📤 Pushing tag to origin..."
git push origin "$VERSION"
echo ""

echo "=========================================="
echo "  ✅ Release $VERSION created successfully!"
echo "=========================================="
echo ""
echo "Next steps:"
echo "1. Create GitHub Release: https://github.com/minzuuniversity/sqlrustgo/releases/new"
echo "2. Update CHANGELOG.md"
echo "3. Announce the release"
