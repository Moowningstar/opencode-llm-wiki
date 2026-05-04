#!/usr/bin/env bash
set -euo pipefail

echo "=== LanceDB to RuVector Migration Script ==="
echo ""

PROJECT_PATH="${1:-.}"
LANCEDB_PATH="$PROJECT_PATH/.llm-wiki/lancedb"
RUVECTOR_PATH="$PROJECT_PATH/.llm-wiki/ruvector.rvf"

if [ ! -d "$LANCEDB_PATH" ]; then
    echo "Error: LanceDB directory not found at $LANCEDB_PATH"
    exit 1
fi

echo "Project path: $PROJECT_PATH"
echo "LanceDB path: $LANCEDB_PATH"
echo "RuVector path: $RUVECTOR_PATH"
echo ""

if [ -f "$RUVECTOR_PATH" ]; then
    echo "Warning: RuVector database already exists at $RUVECTOR_PATH"
    read -p "Do you want to overwrite it? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Migration cancelled."
        exit 0
    fi
    rm -f "$RUVECTOR_PATH"
fi

echo "Building migration tool..."
cargo build --release --bin llm-wiki --features ruvector

echo ""
echo "Running migration..."
./target/release/llm-wiki migrate \
    --from lancedb \
    --to ruvector \
    --project "$PROJECT_PATH"

echo ""
echo "=== Migration Complete ==="
echo ""
echo "Next steps:"
echo "1. Verify the migration: cargo run --release --features ruvector -- check"
echo "2. Test the new backend: cargo test --features ruvector"
echo "3. Backup LanceDB: mv $LANCEDB_PATH ${LANCEDB_PATH}.backup"
echo ""
