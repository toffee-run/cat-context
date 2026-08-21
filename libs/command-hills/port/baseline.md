# Базовый снимок поведения CLI (cat-context)

## Разбор аргументов

| Команда | Код возврата | Первая строка вывода |
|---|---|---|
| `cat-context --help` | 2 | `error: unexpected argument '--help' found` |
| `cat-context -h` | 2 | `error: unexpected argument '-h' found` |
| `cat-context help` | 2 | `error: unrecognized subcommand 'help'` |
| `cat-context --version` | 0 | `cat-context 0.1.0` |
| `cat-context -V` | 0 | `cat-context 0.1.0` |
| `cat-context --unknown` | 2 | `error: unexpected argument '--unknown' found` |
| `cat-context -u` | 2 | `error: unexpected argument '-u' found` |
| `cat-context unknown_subcmd` | 2 | `error: unrecognized subcommand 'unknown_subcmd'` |
| `cat-context --connect invalid://host` | 2 | `error: invalid value 'invalid://host' for '--connect <URL>': URI scheme is not supported: invalid://host` |
| `cat-context start --help` | 2 | `error: unexpected argument '--help' found` |
| `cat-context start -h` | 2 | `error: unexpected argument '-h' found` |
| `cat-context start help` | 2 | `error: unexpected argument 'help' found` |
| `cat-context start --unknown` | 2 | `error: unexpected argument '--unknown' found` |
| `cat-context start -b alpine` | 2 | `error: unexpected argument '-b' found` |
| `cat-context start -a codex` | 2 | `error: unexpected argument '-a' found` |
| `cat-context start -f plan.md` | 2 | `error: unexpected argument '-f' found` |
| `cat-context start --base invalid` | 2 | `error: invalid value 'invalid' for '--base <BASE>'` |
| `cat-context start --agent invalid` | 2 | `error: invalid value 'invalid' for '--agent <AGENT>'` |
| `cat-context start --file plan.txt` | 2 | `error: invalid value 'plan.txt' for '--file <FILE>': нужен .md файл: plan.txt` |
| `cat-context start --file plan.md --text hello` | 2 | `error: the argument '--file <FILE>' cannot be used with '--text <TEXT>'` |
| `cat-context start --file plan.md --no-prompt` | 2 | `error: the argument '--file <FILE>' cannot be used with '--no-prompt'` |
| `cat-context start --text hello --no-prompt` | 2 | `error: the argument '--text <TEXT>' cannot be used with '--no-prompt'` |
| `cat-context start --save` | 2 | `error: unexpected argument '--save' found` |
| `cat-context start --no-save` | 2 | `error: unexpected argument '--no-save' found` |
| `cat-context start --container test` | 2 | `error: unexpected argument '--container' found` |
| `cat-context start --base alpine --agent codex --no-prompt` | 0 | `<empty>` |
| `cat-context start --base debian --agent claude-code --text привет` | 0 | `<empty>` |
| `cat-context start --base arch --agent opencode --file plan.md` | 0 | `<empty>` |
| `cat-context restart --help` | 2 | `error: unexpected argument '--help' found` |
| `cat-context restart -h` | 2 | `error: unexpected argument '-h' found` |
| `cat-context restart help` | 2 | `error: unexpected argument 'help' found` |
| `cat-context restart --unknown` | 2 | `error: unexpected argument '--unknown' found` |
| `cat-context restart -c test` | 2 | `error: unexpected argument '-c' found` |
| `cat-context restart -b debian` | 2 | `error: unexpected argument '-b' found` |
| `cat-context restart -s` | 2 | `error: unexpected argument '-s' found` |
| `cat-context restart --base invalid` | 2 | `error: invalid value 'invalid' for '--base <BASE>'` |
| `cat-context restart --file plan.txt` | 2 | `error: invalid value 'plan.txt' for '--file <FILE>': нужен .md файл: plan.txt` |
| `cat-context restart --file plan.md --text hello` | 2 | `error: the argument '--file <FILE>' cannot be used with '--text <TEXT>'` |
| `cat-context restart --file plan.md --no-prompt` | 2 | `error: the argument '--file <FILE>' cannot be used with '--no-prompt'` |
| `cat-context restart --text hello --no-prompt` | 2 | `error: the argument '--text <TEXT>' cannot be used with '--no-prompt'` |
| `cat-context restart --save --no-save` | 2 | `error: the argument '--save' cannot be used with '--no-save'` |
| `cat-context restart --agent codex` | 2 | `error: unexpected argument '--agent' found` |
| `cat-context restart --container c1 --base alpine --no-prompt --save` | 0 | `<empty>` |
| `cat-context restart --container c1 --base debian --text привет --no-save` | 0 | `<empty>` |
| `cat-context restart --container c1 --base arch --file plan.md --save` | 0 | `<empty>` |
| `cat-context stop --help` | 2 | `error: unexpected argument '--help' found` |
| `cat-context stop -h` | 2 | `error: unexpected argument '-h' found` |
| `cat-context stop help` | 2 | `error: unexpected argument 'help' found` |
| `cat-context stop --unknown` | 2 | `error: unexpected argument '--unknown' found` |
| `cat-context stop -c c1` | 2 | `error: unexpected argument '-c' found` |
| `cat-context stop --base alpine` | 2 | `error: unexpected argument '--base' found` |
| `cat-context stop --agent codex` | 2 | `error: unexpected argument '--agent' found` |
| `cat-context stop --file plan.md` | 2 | `error: unexpected argument '--file' found` |
| `cat-context stop --save` | 2 | `error: unexpected argument '--save' found` |
| `cat-context stop --no-save` | 2 | `error: unexpected argument '--no-save' found` |
| `cat-context stop --container c1` | 0 | `<empty>` |
| `cat-context delete --help` | 2 | `error: unexpected argument '--help' found` |
| `cat-context delete -h` | 2 | `error: unexpected argument '-h' found` |
| `cat-context delete help` | 2 | `error: unexpected argument 'help' found` |
| `cat-context delete --unknown` | 2 | `error: unexpected argument '--unknown' found` |
| `cat-context delete -c c1` | 2 | `error: unexpected argument '-c' found` |
| `cat-context delete --base alpine` | 2 | `error: unexpected argument '--base' found` |
| `cat-context delete --agent codex` | 2 | `error: unexpected argument '--agent' found` |
| `cat-context delete --file plan.md` | 2 | `error: unexpected argument '--file' found` |
| `cat-context delete --save` | 2 | `error: unexpected argument '--save' found` |
| `cat-context delete --no-save` | 2 | `error: unexpected argument '--no-save' found` |
| `cat-context delete --container c1` | 0 | `<empty>` |

## Автодополнение (zsh)

### Подкоманды и глобальные флаги в корне

```bash
_CLAP_IFS=$'\n' _CLAP_COMPLETE_INDEX=1 COMPLETE=zsh cat-context 
```

```
start:запустить новый контейнер
restart:пересоздать контейнер
stop:остановить контейнер
delete:удалить контейнер
--connect
--version:Print version
```

### Флаги подкоманды start

```bash
_CLAP_IFS=$'\n' _CLAP_COMPLETE_INDEX=2 COMPLETE=zsh cat-context start 
```

```
--base
--connect
--agent
--file
--text
--no-prompt
```

### Флаги подкоманды restart

```bash
_CLAP_IFS=$'\n' _CLAP_COMPLETE_INDEX=2 COMPLETE=zsh cat-context restart 
```

```
--container
--connect
--base
--file
--text
--no-prompt
--save
--no-save
```

### Флаги подкоманды stop

```bash
_CLAP_IFS=$'\n' _CLAP_COMPLETE_INDEX=2 COMPLETE=zsh cat-context stop 
```

```
--container
--connect
```

### Флаги подкоманды delete

```bash
_CLAP_IFS=$'\n' _CLAP_COMPLETE_INDEX=2 COMPLETE=zsh cat-context delete 
```

```
--container
--connect
```

### Значения флага --base

```bash
_CLAP_IFS=$'\n' _CLAP_COMPLETE_INDEX=3 COMPLETE=zsh cat-context start --base 
```

```
debian
alpine
arch
```

### Значения флага --agent

```bash
_CLAP_IFS=$'\n' _CLAP_COMPLETE_INDEX=3 COMPLETE=zsh cat-context start --agent 
```

```
claude-code
codex
antigravity
opencode
```

### Значения флага --connect (схемы)

```bash
_CLAP_IFS=$'\n' _CLAP_COMPLETE_INDEX=2 COMPLETE=zsh cat-context --connect 
```

```
unix\://
npipe\://
tcp\://
http\://
https\://
ssh\://
```

### Значения флага --container (без демона Docker)

```bash
_CLAP_IFS=$'\n' _CLAP_COMPLETE_INDEX=3 COMPLETE=zsh cat-context stop --container 
```

```
```

### Значения флага --file (каталог с plan.md, notes.txt, sub/)

```bash
_CLAP_IFS=$'\n' _CLAP_COMPLETE_INDEX=3 COMPLETE=zsh cat-context start --file ""
```

```
.
plan.md
sub/
```

