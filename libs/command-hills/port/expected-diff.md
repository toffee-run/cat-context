# Ожидаемые отличия поведения CLI после порта

Список намеренных изменений поведения, зафиксированных в `decisions.md` (раздел «Намеренные отличия после порта»).

## 1. Снятие валидации с аргумента `--file` (Требование 5)

По требованию 5 тип поля `PathBuf` не накладывает валидацию расширения, а расширение `.md` используется только для подсказок/автодополнения.

| Команда | Было (baseline.md) | Станет после порта | Причина |
|---|---|---|---|
| `cat-context start --file plan.txt` | `2` / `error: invalid value 'plan.txt' for '--file <FILE>': нужен .md файл: plan.txt` | `0` / `<empty>` | Требование 5: PathBuf не валидирует .md |
| `cat-context restart --file plan.txt` | `2` / `error: invalid value 'plan.txt' for '--file <FILE>': нужен .md файл: plan.txt` | `0` / `<empty>` | Требование 5: PathBuf не валидирует .md |

## Точные ожидаемые результаты

```expect
cat-context start --file plan.txt | 0 | <empty>
cat-context restart --file plan.txt | 0 | <empty>
```
