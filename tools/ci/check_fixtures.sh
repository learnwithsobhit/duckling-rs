#!/usr/bin/env bash
# Fail if fixtures/*.jsonl changed without being named in DIVERGENCES.md.
#
# Usage:
#   tools/ci/check_fixtures.sh [BASE_SHA] [HEAD_SHA]
#
# Defaults: BASE is origin/master (or the first parent of HEAD), HEAD is HEAD.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

HEAD="${2:-HEAD}"
if [ -n "${1:-}" ]; then
  BASE="$1"
elif git rev-parse --verify origin/master >/dev/null 2>&1; then
  BASE="origin/master"
else
  BASE="$(git rev-parse HEAD^ 2>/dev/null || git rev-parse HEAD)"
fi

# Empty before-sha on a new branch push.
if [ "$BASE" = "0000000000000000000000000000000000000000" ]; then
  echo "check_fixtures: no base commit; skip"
  exit 0
fi

changed="$(git diff --name-only "$BASE" "$HEAD" -- 'fixtures/*.jsonl' || true)"
if [ -z "$changed" ]; then
  echo "check_fixtures: no fixture files changed"
  exit 0
fi

div="$ROOT/DIVERGENCES.md"
if [ ! -f "$div" ]; then
  echo "check_fixtures: DIVERGENCES.md missing but fixtures changed:" >&2
  echo "$changed" >&2
  exit 1
fi

fail=0
while IFS= read -r f; do
  [ -z "$f" ] && continue
  base="$(basename "$f")"
  if grep -F -q -- "$f" "$div" || grep -F -q -- "$base" "$div"; then
    echo "check_fixtures: $f listed in DIVERGENCES.md"
  else
    echo "check_fixtures: $f changed but is not named in DIVERGENCES.md" >&2
    fail=1
  fi
done <<EOF
$changed
EOF

if [ "$fail" -ne 0 ]; then
  echo "check_fixtures: add the fixture filename to DIVERGENCES.md (Phase 7 allowlist) before changing gold files." >&2
  exit 1
fi
