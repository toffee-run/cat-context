#!/usr/bin/env bash
set -euo pipefail

cargo build --bin cat-context --quiet

BIN="$(pwd)/target/debug/cat-context"

run_cmd() {
    local cmd=("$@")
    local output
    local status=0
    if command -v setsid >/dev/null 2>&1; then
        output=$(setsid -w "$BIN" "${cmd[@]}" < /dev/null 2>&1) || status=$?
    else
        output=$("$BIN" "${cmd[@]}" < /dev/null 2>&1) || status=$?
    fi
    local first_line
    first_line=$(echo "$output" | head -n 1)
    if [ -z "$first_line" ]; then
        first_line="<empty>"
    fi
    printf "| \`cat-context %s\` | %d | \`%s\` |\n" "${cmd[*]}" "$status" "$first_line"
}

run_completion() {
    local title="$1"
    local idx="$2"
    shift 2
    local cmd=("$@")
    echo "### $title"
    echo ""
    echo '```bash'
    echo "_CLAP_IFS=\$'\n' _CLAP_COMPLETE_INDEX=$idx COMPLETE=zsh cat-context ${cmd[*]}"
    echo '```'
    echo ""
    echo '```'
    local comp_out
    comp_out=$(_CLAP_IFS=$'\n' _CLAP_COMPLETE_INDEX="$idx" COMPLETE=zsh "$BIN" -- cat-context "${cmd[@]}" < /dev/null 2>&1 || true)
    if [ -n "$comp_out" ]; then
        echo "$comp_out"
    fi
    echo '```'
    echo ""
}

run_connect_completion() {
    local title="$1"
    local idx="$2"
    shift 2
    local cmd=("$@")
    echo "### $title"
    echo ""
    echo '```bash'
    echo "_CLAP_IFS=\$'\n' _CLAP_COMPLETE_INDEX=$idx COMPLETE=zsh cat-context ${cmd[*]}"
    echo '```'
    echo ""
    echo '```'
    local comp_out
    comp_out=$(_CLAP_IFS=$'\n' _CLAP_COMPLETE_INDEX="$idx" COMPLETE=zsh "$BIN" -- cat-context "${cmd[@]}" < /dev/null 2>&1 | grep -E '^(unix|npipe|tcp|http|https|ssh)\\://$' || true)
    if [ -n "$comp_out" ]; then
        echo "$comp_out"
    fi
    echo '```'
    echo ""
}

run_file_completion() {
    local title="$1"
    local fixture_dir="$(pwd)/target/file_completion_fixture"
    rm -rf "$fixture_dir"
    mkdir -p "$fixture_dir/sub"
    touch "$fixture_dir/plan.md" "$fixture_dir/notes.txt"
    echo "### $title"
    echo ""
    echo '```bash'
    echo "_CLAP_IFS=\$'\n' _CLAP_COMPLETE_INDEX=3 COMPLETE=zsh cat-context start --file \"\""
    echo '```'
    echo ""
    echo '```'
    local comp_out
    comp_out=$(cd "$fixture_dir" && _CLAP_IFS=$'\n' _CLAP_COMPLETE_INDEX=3 COMPLETE=zsh "$BIN" -- cat-context start --file "" < /dev/null 2>&1 || true)
    if [ -n "$comp_out" ]; then
        echo "$comp_out"
    fi
    echo '```'
    echo ""
    rm -rf "$fixture_dir"
}

echo "# Базовый снимок поведения CLI (cat-context)"
echo ""
echo "## Разбор аргументов"
echo ""
echo "| Команда | Код возврата | Первая строка вывода |"
echo "|---|---|---|"

run_cmd --help
run_cmd -h
run_cmd help
run_cmd --version
run_cmd -V
run_cmd --unknown
run_cmd -u
run_cmd unknown_subcmd
run_cmd --connect invalid://host

run_cmd start --help
run_cmd start -h
run_cmd start help
run_cmd start --unknown
run_cmd start -b alpine
run_cmd start -a codex
run_cmd start -f plan.md
run_cmd start --base invalid
run_cmd start --agent invalid
run_cmd start --file plan.txt
run_cmd start --file plan.md --text hello
run_cmd start --file plan.md --no-prompt
run_cmd start --text hello --no-prompt
run_cmd start --save
run_cmd start --no-save
run_cmd start --container test
run_cmd start --base alpine --agent codex --no-prompt
run_cmd start --base debian --agent claude-code --text привет
run_cmd start --base arch --agent opencode --file plan.md

run_cmd restart --help
run_cmd restart -h
run_cmd restart help
run_cmd restart --unknown
run_cmd restart -c test
run_cmd restart -b debian
run_cmd restart -s
run_cmd restart --base invalid
run_cmd restart --file plan.txt
run_cmd restart --file plan.md --text hello
run_cmd restart --file plan.md --no-prompt
run_cmd restart --text hello --no-prompt
run_cmd restart --save --no-save
run_cmd restart --agent codex
run_cmd restart --container c1 --base alpine --no-prompt --save
run_cmd restart --container c1 --base debian --text привет --no-save
run_cmd restart --container c1 --base arch --file plan.md --save

run_cmd stop --help
run_cmd stop -h
run_cmd stop help
run_cmd stop --unknown
run_cmd stop -c c1
run_cmd stop --base alpine
run_cmd stop --agent codex
run_cmd stop --file plan.md
run_cmd stop --save
run_cmd stop --no-save
run_cmd stop --container c1

run_cmd delete --help
run_cmd delete -h
run_cmd delete help
run_cmd delete --unknown
run_cmd delete -c c1
run_cmd delete --base alpine
run_cmd delete --agent codex
run_cmd delete --file plan.md
run_cmd delete --save
run_cmd delete --no-save
run_cmd delete --container c1

echo ""
echo "## Автодополнение (zsh)"
echo ""
run_completion "Подкоманды и глобальные флаги в корне" 1 ""
run_completion "Флаги подкоманды start" 2 start ""
run_completion "Флаги подкоманды restart" 2 restart ""
run_completion "Флаги подкоманды stop" 2 stop ""
run_completion "Флаги подкоманды delete" 2 delete ""
run_completion "Значения флага --base" 3 start --base ""
run_completion "Значения флага --agent" 3 start --agent ""
run_connect_completion "Значения флага --connect (схемы)" 2 --connect ""
run_completion "Значения флага --container (без демона Docker)" 3 stop --container ""
run_file_completion "Значения флага --file (каталог с plan.md, notes.txt, sub/)"

