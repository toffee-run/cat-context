# Ожидаемые отличия поведения CLI после порта

Список намеренных изменений поведения, зафиксированных в `decisions.md` (раздел «Намеренные отличия после порта»).

## 1. Снятие валидации с аргумента `--file` (Требование 5)

По требованию 5 тип поля `PathBuf` не накладывает валидацию расширения, а расширение `.md` используется только для подсказок/автодополнения.

| Команда | Было (baseline.md) | Станет после порта | Причина |
|---|---|---|---|
| `cat-context start --file plan.txt` | `2` / `error: invalid value 'plan.txt' for '--file <FILE>': нужен .md файл: plan.txt` | `1` / `The input device is not a TTY` | Валидация снята (требование 5): аргумент принимается clap, команда переходит к заполнению недостающих полей и останавливается на диалоге без TTY |
| `cat-context restart --file plan.txt` | `2` / `error: invalid value 'plan.txt' for '--file <FILE>': нужен .md файл: plan.txt` | `1` / `The input device is not a TTY` | Валидация снята (требование 5): аргумент принимается clap, команда переходит к заполнению недостающих полей и останавливается на диалоге без TTY |

## Точные ожидаемые результаты

```expect
cat-context start --file plan.txt | 1 | The input device is not a TTY
cat-context restart --file plan.txt | 1 | The input device is not a TTY
```
