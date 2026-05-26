#!/usr/bin/env bash
# Embed markdown docs into docs.html for offline / file:// support
# Usage: bash build.sh

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

INPUT="$SCRIPT_DIR/docs.html"
OUTPUT="$SCRIPT_DIR/docs.html"

# Read docs and escape for JS string literals
escape_for_js() {
  sed -e 's/\\/\\\\/g' -e 's/`/\\`/g' -e 's/\$/\\$/g' "$1"
}

GUIDE=$(escape_for_js "$PROJECT_DIR/docs/guide.md")
GUIDE_EN=$(escape_for_js "$PROJECT_DIR/docs/guide.en.md")
ARCH=$(escape_for_js "$PROJECT_DIR/docs/phase8-architecture.md")
PRD=$(escape_for_js "$PROJECT_DIR/docs/phase8-prd.md")

# Build the replacement JS object with heredoc-style embedding
# We use a temp file approach to handle large content
python3 -c "
import sys

with open('$OUTPUT', 'r') as f:
    html = f.read()

# Read markdown files
def read_and_escape(path):
    with open(path, 'r') as f:
        content = f.read()
    # Escape backticks and backslash for JS template literal
    content = content.replace('\\\\', '\\\\\\\\')
    content = content.replace('\`', '\\\\\`')
    content = content.replace('\${', '\\\${')
    return content

guide = read_and_escape('$PROJECT_DIR/docs/guide.md')
guide_en = read_and_escape('$PROJECT_DIR/docs/guide.en.md')
arch = read_and_escape('$PROJECT_DIR/docs/phase8-architecture.md')
prd = read_and_escape('$PROJECT_DIR/docs/phase8-prd.md')

# Replace the inline placeholders
html = html.replace(
    'INLINE_DOCS = {\n      INLINE_GUID: null,\n      INLINE_GUIDE_EN: null,\n      INLINE_ARCH: null,\n      INLINE_PRD: null,\n    };',
    '''INLINE_DOCS = {
      INLINE_GUID: \`''' + guide + '''\`,
      INLINE_GUIDE_EN: \`''' + guide_en + '''\`,
      INLINE_ARCH: \`''' + arch + '''\`,
      INLINE_PRD: \`''' + prd + '''\`,
    };''')

with open('$OUTPUT', 'w') as f:
    f.write(html)

print('OK: docs embedded into docs.html')
"
