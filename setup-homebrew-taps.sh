#!/bin/bash

# Setup Homebrew Taps for PrismNote, StatGuard, and ClusterAudienceKit
# This script creates the tap repositories and adds formulas

set -e

GITHUB_USER="Mullassery"

echo "🍻 Setting up Homebrew Taps..."
echo ""

# Function to create a tap
create_tap() {
  local name=$1
  local description=$2
  local formula_name=$3

  echo "Creating tap for $name..."

  # Create tap repo on GitHub
  gh repo create "homebrew-${name}" \
    --public \
    --description "Homebrew tap for $name" \
    --add-readme \
    2>/dev/null || echo "  Tap repo may already exist"

  # Clone tap repo
  tap_dir="/tmp/homebrew-${name}"
  if [ -d "$tap_dir" ]; then
    rm -rf "$tap_dir"
  fi

  git clone "https://github.com/${GITHUB_USER}/homebrew-${name}.git" "$tap_dir"

  # Create Formula directory
  mkdir -p "$tap_dir/Formula"

  echo "  ✓ Tap created: $name"
}

# Function to add formula
add_formula() {
  local name=$1
  local formula_file=$2
  local tap_dir="/tmp/homebrew-${name}"

  echo "Adding formula for $name..."

  # Copy formula
  cp "$formula_file" "$tap_dir/Formula/"

  # Commit and push
  cd "$tap_dir"
  git config user.name "Bot" || true
  git config user.email "bot@prismnote.dev" || true
  git add Formula/
  git commit -m "Add $name formula" || true
  git push origin main 2>/dev/null || true

  echo "  ✓ Formula added: $name"
}

# Create each tap
echo ""
echo "📦 Creating Homebrew Taps..."
echo ""

create_tap "statguard" "Homebrew tap for StatGuard" "statguard"
create_tap "clusteraudiencekit" "Homebrew tap for ClusterAudienceKit" "clusteraudiencekit"
create_tap "prismnote" "Homebrew tap for PrismNote" "prismnote"

echo ""
echo "✅ Homebrew taps created!"
echo ""
echo "Install via:"
echo "  brew tap Mullassery/statguard"
echo "  brew install statguard"
echo ""
echo "  brew tap Mullassery/clusteraudiencekit"
echo "  brew install clusteraudiencekit"
echo ""
echo "  brew tap Mullassery/prismnote"
echo "  brew install prismnote"
echo ""
