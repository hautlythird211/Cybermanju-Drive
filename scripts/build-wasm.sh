#!/bin/bash
set -e

echo "=== Building Cybermanju Drive WASM module ==="

# Navigate to drive-wasm crate
cd "$(dirname "$0")/../crates/drive-wasm"

echo ""
echo "Step 1: Building WASM crate with wasm-pack..."
wasm-pack build --target web --out-dir ../../node_modules/cybermanju-drive-wasm

echo ""
echo "Step 2: Building Vite project with WASM config..."
cd ../../
npx vite build --mode production --config vite.config.wasm.ts --outDir dist-wasm

echo ""
echo "=== Done! ==="
echo "WASM output: dist-wasm/"
echo "To serve locally: npx vite preview --port 4175 --outDir dist-wasm"
