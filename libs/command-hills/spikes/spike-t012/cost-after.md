# T-012 Стоимость добавления параметра после порта

Гарантия компилятора: **полная** (добавить аргумент в CLI, забыв добавить его в команду `Action`, теперь синтаксически невозможно, так как `StartArgs` и диалоги генерируются из самого `Action`).

---

## Места правки

Количество файлов: **2** (`src/lib.rs`, `src/cli.rs`).
Количество мест правки: **6** (вместо 8 до порта).

| № | Файл | Место | Что меняется |
|---|---|---|---|
| 1 | `src/lib.rs` | Определение типов | Объявление `enum Memory` с derive (`ValueEnum` и т.д.). Ручная реализация `Display` больше не нужна! |
| 2 | `src/lib.rs` | `Action::Start` | Добавление целевого поля `memory: Memory` с атрибутом `#[hill(ask = ...)]` |
| 3 | `src/cli.rs` | Импорты (тесты) | Добавление `Memory` в импорты для тестов (`use crate::{..., Memory}`) |
| 4 | `src/cli.rs` | Тесты парсинга | Проверка опциональности в `every_start_argument_is_optional` |
| 5 | `src/cli.rs` | Тесты заполнения | Обновление паттерна и утверждений в `full_start_arguments_need_no_questions` |
| 6 | `src/cli.rs` | Тесты заполнения | Обновление паттерна в `start_accepts_a_markdown_prompt` |

---

## Поведение компилятора при пропуске шагов

### 1. Пропущено объявление `enum Memory` (шаг 1)
- Команда: `cargo check`
- Код ошибки: `E0425`
- Первая строка: `error[E0425]: cannot find type \`Memory\` in this scope`
- Имя поля видно: **да** (указывает прямо на строку `memory: Memory`).

### 2. Добавлен аргумент без обновления целевого типа (Случай «тихой потери»)
До порта можно было добавить поле в `StartArgs`, но забыть передать его в `Action::Start`. Сейчас `StartArgs` генерируется макросом. 
Добавить аргумент в CLI, не добавив его в `Action`, **физически невозможно**, так как ручной структуры больше нет. Если разработчик попытается проверить поле в тесте (шаги 4-6), не добавив его в `Action`, произойдёт следующее:
- Команда: `cargo check --all-targets`
- Код ошибки: `E0609` (и `E0026`)
- Первая строка: `error[E0609]: no field \`memory\` on type \`StartArgs\``
- Имя поля видно: **да**.
- Вывод: проблема тихой потери параметра полностью решена. Единым источником правды стал `Action`.

---

## Дифф добавления параметра

```diff
diff --git a/src/cli.rs b/src/cli.rs
index eb1e4eb..c36d2e4 100644
--- a/src/cli.rs
+++ b/src/cli.rs
@@ -128,6 +128,7 @@ mod argument_tests {
 
         assert!(args.base.is_none());
         assert!(args.agent.is_none());
+        assert!(args.memory.is_none());
         assert!(args.prompt.file.is_none());
         assert!(args.prompt.text.is_none());
         assert!(!args.prompt.no_prompt);
@@ -370,7 +371,7 @@ mod fill_tests {
 mod fill_tests {
     use super::fixtures::offline_docker;
     use super::*;
-    use crate::{Action, Agent, Base};
+    use crate::{Action, Agent, Base, Memory};
 
     async fn action_from(args: &[&str]) -> Action {
         let action = Cli::try_parse_from(args)
@@ -391,6 +392,8 @@ mod fill_tests {
             "alpine",
             "--agent",
             "codex",
+            "--memory",
+            "off",
             "--text",
             "привет",
         ])
@@ -398,6 +401,7 @@ mod fill_tests {
         let Action::Start {
             base,
             agent,
+            memory,
             prompt,
         } = action
         else {
@@ -405,6 +409,7 @@ mod fill_tests {
 
         assert_eq!(base, Base::Alpine);
         assert_eq!(agent, Agent::Codex);
+        assert_eq!(memory, Memory::Off);
         assert!(matches!(prompt, Prompt::Text(text) if text == "привет"));
     }
 
@@ -416,6 +421,8 @@ mod fill_tests {
             "arch",
             "--agent",
             "opencode",
+            "--memory",
+            "shared",
             "--file",
             "plan.md",
         ])
@@ -423,7 +430,7 @@ mod fill_tests {
-        let Action::Start { prompt, .. } = action else {
+        let Action::Start { prompt, memory, .. } = action else {
             panic!("ожидался Start");
         };
 
         assert!(matches!(prompt, Prompt::File(path) if path == Path::new("plan.md")));
+        assert_eq!(memory, Memory::Shared);
     }
diff --git a/src/lib.rs b/src/lib.rs
index b34d0b1..8fb0650 100644
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -25,6 +25,13 @@ pub enum Agent {
     Opencode,
 }
 
+#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, ValueEnum)]
+pub enum Memory {
+    #[default]
+    Off,
+    Shared,
+}
+
 #[derive(Clone, Debug, Default)]
 pub enum Prompt {
     File(PathBuf),
@@ -41,6 +48,8 @@ pub enum Action {
         base: Base,
         #[hill(ask = "Агент", arg(long, value_enum, value_name = "AGENT"))]
         agent: Agent,
+        #[hill(ask = "Память", arg(long, value_enum, value_name = "MEMORY"))]
+        memory: Memory,
         #[hill(args = cli::PromptArgs)]
         prompt: Prompt,
     },
```
