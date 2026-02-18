#!/bin/bash

set -e

TYPE=$1

if [ -z "$TYPE" ]; then
  echo "Usage: ./scripts/bump_version.sh [major|minor|patch]"
  echo ""
  echo "Examples:"
  echo "  ./scripts/bump_version.sh patch  # v1.0.0 -> v1.0.1"
  echo "  ./scripts/bump_version.sh minor  # v1.0.0 -> v1.1.0"
  echo "  ./scripts/bump_version.sh major  # v1.0.0 -> v2.0.0"
  exit 1
fi

echo "=========================================="
echo "  SQLRustGo Version Bump Script"
echo "=========================================="
echo ""

echo "🔍 Getting current version..."
CURRENT=$(git describe --tags --abbrev=0 2>/dev/null || echo "v0.0.0")
echo "Current version: $CURRENT"

IFS='.' read -r MAJOR MINOR PATCH <<< "${CURRENT#v}"

echo ""
echo "📊 Current version components:"
echo "  MAJOR: $MAJOR"
echo "  MINOR: $MINOR"
echo "  PATCH: $PATCH"
echo ""

case $TYPE in
  major)
    MAJOR=$((MAJOR+1))
    MINOR=0
    PATCH=0
    echo "📈 Bumping MAJOR version"
    ;;
  minor)
    MINOR=$((MINOR+1))
    PATCH=0
    echo "📈 Bumping MINOR version"
    ;;
  patch)
    PATCH=$((PATCH+1))
    echo "📈 Bumping PATCH version"
    ;;
  *)
    echo "❌ Invalid type: $TYPE"
    echo "Valid types: major, minor, patch"
    exit 1
    ;;
esac

NEW_VERSION="v$MAJOR.$MINOR.$PATCH"

echo ""
echo "✨ New version: $NEW_VERSION"
echo ""

read -p "Create tag $NEW_VERSION? (y/n) " -n 1 -r
echo ""

if [[ ! $REPLY =~ ^[Yy]$ ]]; then
  echo "❌ Cancelled"
  exit 1
fi

echo ""
echo "🏷️  Creating tag $NEW_VERSION..."
git tag -a "$NEW_VERSION" -m "Release $NEW_VERSION"

echo ""
echo "📤 Pushing tag to origin..."
git push origin "$NEW_VERSION"

echo ""
echo "=========================================="
echo "  ✅ Version bumped to $NEW_VERSION"
echo "=========================================="
echo ""
echo "Next steps:"
echo "1. Update CHANGELOG.md"
echo "2. Create GitHub Release"
echo "3. Announce the release"
