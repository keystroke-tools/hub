#!/usr/bin/env bash

set -euo pipefail

echo "[INFO] Building all workspace crates for wasm32"
cargo build --release --target wasm32-unknown-unknown

echo "[INFO] Finding all .wasm files"
echo ""
find ./target/wasm32-unknown-unknown/release -maxdepth 1 -type f -name '*.wasm' | while read -r wasm_file; do
    wasm_filename=$(basename "$wasm_file")
    wasm_name="${wasm_filename%.wasm}"

    # Step 1: Try to find the crate path based on filename
    # Assumes: crate target name == crate dir name (adjust if needed)
    crate_dir=$(find . -maxdepth 1 -type d -name "$wasm_name" -print -quit)

    if [ -z "$crate_dir" ]; then
        echo "[ERROR] Could not find crate directory for $wasm_filename, skipping."
        continue
    fi

    output_dir="$crate_dir/output"
    mkdir -p "$output_dir"

    pre_opt="$output_dir/pre-opt.wasm"
    final="$output_dir/plugin.wasm"
    checksum="$output_dir/plugin.wasm.sha256"

    echo "[INFO] Processing $wasm_filename in $crate_dir"

    # Copy and optimize
    cp "$wasm_file" "$pre_opt"
    echo "[INFO] Optimizing with wasm-opt"
    wasm-opt -Oz "$pre_opt" -o "$final"
    rm "$pre_opt"

    # Generate checksum
    echo "[INFO] Generating checksum"
    sha256sum -b "$final" | cut -d' ' -f1 | tr -d '\n' > "$checksum"

    echo "[SUCCESS] Output in $final"
    echo "[INFO] Checksum: $(cat $checksum)"
    echo ""
done

echo "[SUCCESS] All .wasm files processed!"
