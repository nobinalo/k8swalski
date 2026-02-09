#!/usr/bin/env bash
set -euo pipefail

# Generate help output from the binary
HELP_OUTPUT=$(cargo run --quiet -- --help 2>/dev/null || true)

# Check if we got output
if [ -z "$HELP_OUTPUT" ]; then
    echo "Error: Failed to generate help output"
    exit 1
fi

# Create temporary file
TEMP_FILE=$(mktemp)
README_FILE="README.md"

# Read the README and replace content between markers
awk -v help="$HELP_OUTPUT" '
BEGIN { in_block = 0 }
/<!-- BEGIN_CLI_HELP -->/ {
    print
    print "```"
    print help
    print "```"
    in_block = 1
    next
}
/<!-- END_CLI_HELP -->/ {
    in_block = 0
}
!in_block { print }
' "$README_FILE" > "$TEMP_FILE"

# Replace original file
mv "$TEMP_FILE" "$README_FILE"

echo "README.md updated successfully"
