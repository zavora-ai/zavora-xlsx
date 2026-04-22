#!/bin/bash
# CLI Tool Demo — demonstrates inspect, export, and convert subcommands
# Run from the workspace root: bash zavora-xlsx-cli/examples/cli_demo.sh

set -e

echo "=== zavora-xlsx CLI Demo ==="
echo ""

# First, create a test xlsx file using a small Rust helper
echo "1. Creating test workbook..."
cargo run --example derive_macro_demo 2>/dev/null
echo ""

# Inspect the file
echo "2. Inspecting the workbook:"
cargo run -p zavora-xlsx-cli -- inspect output/derive_macro_demo.xlsx
echo ""

# Export to CSV
echo "3. Exporting Sheet1 to CSV:"
cargo run -p zavora-xlsx-cli -- export output/derive_macro_demo.xlsx
echo ""

# Export to TSV
echo "4. Exporting as TSV:"
cargo run -p zavora-xlsx-cli -- export output/derive_macro_demo.xlsx --tsv
echo ""

# Convert xlsx to CSV files
echo "5. Converting xlsx to CSV files:"
mkdir -p output/csv_demo
cargo run -p zavora-xlsx-cli -- convert output/derive_macro_demo.xlsx --format csv --output output/csv_demo
echo "   Files created:"
ls -la output/csv_demo/
echo ""

echo "=== Demo complete ==="
