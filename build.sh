#!/bin/bash
set -e

echo "🔨 Building PrismNote..."

# Build frontend
echo "📦 Building React frontend..."
cd frontend
npm install
npm run build
cd ..

# Build Rust backend
echo "🦀 Building Rust backend..."
cargo build --release

# Move dist to static serving location
echo "📁 Setting up static assets..."
mkdir -p crates/server/dist
cp -r frontend/dist/* crates/server/dist/

echo "✅ Build complete!"
echo "   Binary: ./target/release/prismnote"
echo "   Run with: ./target/release/prismnote"
