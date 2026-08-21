#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BASELINE_SH="$SCRIPT_DIR/baseline.sh"
BASELINE_MD="$SCRIPT_DIR/baseline.md"
EXPECTED_DIFF_MD="$SCRIPT_DIR/expected-diff.md"

TEMP_SNAPSHOT=$(mktemp)
trap 'rm -f "$TEMP_SNAPSHOT"' EXIT

"$BASELINE_SH" > "$TEMP_SNAPSHOT"

python3 - "$BASELINE_MD" "$TEMP_SNAPSHOT" "$EXPECTED_DIFF_MD" << 'PYEOF'
import sys
import difflib
import re

baseline_path, current_path, expected_path = sys.argv[1], sys.argv[2], sys.argv[3]

with open(baseline_path, "r", encoding="utf-8") as f:
    baseline_lines = f.readlines()

with open(current_path, "r", encoding="utf-8") as f:
    current_lines = f.readlines()

expected_patterns = []
with open(expected_path, "r", encoding="utf-8") as f:
    in_pattern_block = False
    for line in f:
        stripped = line.strip()
        if stripped == "```pattern":
            in_pattern_block = True
            continue
        elif stripped == "```" and in_pattern_block:
            in_pattern_block = False
            continue
        if in_pattern_block and stripped:
            expected_patterns.append(stripped)

matcher = difflib.SequenceMatcher(None, baseline_lines, current_lines)

expected_diffs = []
unexpected_diffs = []

for tag, i1, i2, j1, j2 in matcher.get_opcodes():
    if tag == "equal":
        continue

    chunk_baseline = baseline_lines[i1:i2]
    chunk_current = current_lines[j1:j2]

    diff_text = []
    for line in chunk_baseline:
        diff_text.append(f"- {line.rstrip()}")
    for line in chunk_current:
        diff_text.append(f"+ {line.rstrip()}")

    combined = "\n".join(diff_text)

    is_expected = False
    for pattern in expected_patterns:
        if pattern in combined:
            is_expected = True
            break

    if is_expected:
        expected_diffs.append(combined)
    else:
        unexpected_diffs.append(combined)

print("# Сверка поведения CLI с базовым снимком")
print()
print("## Ожидаемые отличия")
print()
if expected_diffs:
    for item in expected_diffs:
        print("```diff")
        print(item)
        print("```")
        print()
else:
    print("нет")
    print()

print("## Неожиданные отличия")
print()
if unexpected_diffs:
    for item in unexpected_diffs:
        print("```diff")
        print(item)
        print("```")
        print()
    print("Вердикт: обнаружены неожиданные отличия")
    sys.exit(1)
else:
    print("нет")
    print()
    print("Вердикт: поведение полностью соответствует ожиданиям")
    sys.exit(0)
PYEOF
