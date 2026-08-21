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
import re

baseline_path, current_path, expected_path = sys.argv[1], sys.argv[2], sys.argv[3]

def parse_snapshot(path):
    table = {}
    completion_text = ""
    in_table = False
    in_completion = False

    table_pattern = re.compile(r"^\|\s*`([^`]+)`\s*\|\s*(\d+)\s*\|\s*`([^`]*)`\s*\|$")

    with open(path, "r", encoding="utf-8") as f:
        for line in f:
            if line.startswith("## Разбор аргументов"):
                in_table = True
                in_completion = False
                continue
            elif line.startswith("## Автодополнение"):
                in_table = False
                in_completion = True
                continue

            if in_table:
                m = table_pattern.match(line.strip())
                if m:
                    cmd = m.group(1).strip()
                    status = int(m.group(2).strip())
                    output = m.group(3).strip()
                    table[cmd] = (status, output)
            elif in_completion:
                completion_text += line

    return table, completion_text

def parse_expected(path):
    expected = {}
    in_expect_block = False

    with open(path, "r", encoding="utf-8") as f:
        for line in f:
            stripped = line.strip()
            if stripped == "```expect":
                in_expect_block = True
                continue
            elif stripped == "```" and in_expect_block:
                in_expect_block = False
                continue

            if in_expect_block and stripped:
                parts = [p.strip() for p in stripped.split("|")]
                if len(parts) >= 3:
                    cmd = parts[0]
                    status = int(parts[1])
                    output = parts[2]
                    expected[cmd] = (status, output)

    return expected

baseline_table, baseline_comp = parse_snapshot(baseline_path)
current_table, current_comp = parse_snapshot(current_path)
expected_table = parse_expected(expected_path)

expected_diffs = []
unexpected_diffs = []

all_cmds = list(baseline_table.keys())
for cmd in current_table.keys():
    if cmd not in all_cmds:
        all_cmds.append(cmd)

for cmd in all_cmds:
    if cmd in baseline_table and cmd in current_table:
        b_status, b_out = baseline_table[cmd]
        c_status, c_out = current_table[cmd]

        if (b_status, b_out) != (c_status, c_out):
            if cmd in expected_table:
                exp_status, exp_out = expected_table[cmd]
                if (c_status, c_out) == (exp_status, exp_out):
                    expected_diffs.append(
                        f"| `{cmd}` |\n"
                        f"  было:   {b_status} | `{b_out}`\n"
                        f"  стало:  {c_status} | `{c_out}`\n"
                        f"  вердикт: совпадает с ожидаемым результатом ({exp_status} | `{exp_out}`)"
                    )
                else:
                    unexpected_diffs.append(
                        f"| `{cmd}` |\n"
                        f"  было:       {b_status} | `{b_out}`\n"
                        f"  стало:      {c_status} | `{c_out}`\n"
                        f"  ожидалось:  {exp_status} | `{exp_out}`"
                    )
            else:
                unexpected_diffs.append(
                    f"| `{cmd}` |\n"
                    f"  было:   {b_status} | `{b_out}`\n"
                    f"  стало:  {c_status} | `{c_out}`"
                )
    elif cmd in baseline_table and cmd not in current_table:
        b_status, b_out = baseline_table[cmd]
        unexpected_diffs.append(f"Команда пропала из снимка: `{cmd}` ({b_status} | `{b_out}`)")
    elif cmd not in baseline_table and cmd in current_table:
        c_status, c_out = current_table[cmd]
        unexpected_diffs.append(f"Новая необъявленная команда в снимке: `{cmd}` ({c_status} | `{c_out}`)")

if baseline_comp != current_comp:
    unexpected_diffs.append(
        "Расхождение в блоках автодополнения (zsh):\n"
        "Разделы комплишена в текущем снимке отличаются от baseline.md"
    )

print("# Сверка поведения CLI с базовым снимком")
print()
print("## Ожидаемые отличия")
print()
if expected_diffs:
    for item in expected_diffs:
        print(f"- {item}")
    print()
else:
    print("нет")
    print()

print("## Неожиданные отличия")
print()
if unexpected_diffs:
    for item in unexpected_diffs:
        print(f"- {item}")
    print()
    print("Вердикт: обнаружены неожиданные отличия")
    sys.exit(1)
else:
    print("нет")
    print()
    print("Вердикт: поведение полностью соответствует ожиданиям")
    sys.exit(0)
PYEOF
