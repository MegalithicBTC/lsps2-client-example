#!/bin/bash

# Script to concatenate all code files into /tmp/all_code.txt with file paths
# for LLM consumption

OUTPUT_FILE="./tmp/all_code.txt"

# Clear the output file
> "$OUTPUT_FILE"

echo "Concatenating code files to $OUTPUT_FILE..."

# Function to add file with header
add_file() {
    local file="$1"
    echo "" >> "$OUTPUT_FILE"
    echo "=== FILE: $file ===" >> "$OUTPUT_FILE"
    echo "" >> "$OUTPUT_FILE"
    cat "$file" >> "$OUTPUT_FILE"
    echo "" >> "$OUTPUT_FILE"
}

# Add Cargo.toml files
find . -name "Cargo.toml" -not -path "./target/*" | while read -r file; do
    add_file "$file"
done

# Add Cargo.lock
if [ -f "Cargo.lock" ]; then
    add_file "Cargo.lock"
fi

# Add Rust source files
find . -name "*.rs" -not -path "./target/*" | while read -r file; do
    add_file "$file"
done

# Add shell scripts
find . -name "*.sh" -not -path "./target/*" | while read -r file; do
    add_file "$file"
done

# Add README and other documentation
find . -name "README*" -not -path "./target/*" | while read -r file; do
    add_file "$file"
done

# Add rustfmt.toml if present
if [ -f "rustfmt.toml" ]; then
    add_file "rustfmt.toml"
fi

echo "Done! All code files have been concatenated to $OUTPUT_FILE"
echo "File size: $(wc -c < "$OUTPUT_FILE") bytes"
echo "Line count: $(wc -l < "$OUTPUT_FILE") lines"