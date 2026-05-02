#!/bin/bash
set -e

echo "🦀 Building server..."
cd src-server
cargo build --release --bin llm-wiki-server
cargo build --release --bin llm-wiki

echo ""
echo "✅ Server built successfully!"
echo "   API Server: target/release/llm-wiki-server"
echo "   CLI: target/release/llm-wiki"
