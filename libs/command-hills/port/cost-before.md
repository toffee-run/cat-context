# Стоимость добавления параметра до внедрения command-hills

Замер выполнен на примере добавления нового параметра `memory` (enum `Memory` с вариантами `off` и `shared`) в команду `start`.

## Итоговые показатели

- Файлов для правки: **2** (`src/lib.rs`, `src/cli.rs`).
- Мест правки в основном коде: **5** (плюс импорт и тесты).
- Размер диффа (с тестами): **121 строка** (+59 / -11).
- Гарантия компилятора: **частичная** (пропуск `StartArgs` при наличии поля в `Action` ловится компилятором, но добавление аргумента в `StartArgs` без `Action::Start` и `fn start` компилируется без ошибок и предупреждений — параметр тихо теряется во время выполнения).

---

## Места правки

| № | Файл | Место | Что меняется |
|---|---|---|---|
| 1 | `src/lib.rs` | Определение типов | Объявление `enum Memory` с derive (`Clone, Copy, PartialEq, Eq, Debug, Default, ValueEnum`) |
| 2 | `src/lib.rs` | Трейты отображения | Реализация `fmt::Display for Memory` (через `to_possible_value().get_name()`) |
| 3 | `src/lib.rs` | `Action::Start` | Добавление целевого поля `memory: Memory` в структуру варианта `Action::Start` |
| 4 | `src/cli.rs` | Импорты | Добавление `Memory` в `use crate::{...}` |
| 5 | `src/cli.rs` | `StartArgs` | Добавление поля `memory: Option<Memory>` с атрибутом `#[arg(long, value_enum, value_name = "MEMORY")]` |
| 6 | `src/cli.rs` | `fn start` | Интерактивный запрос/разрешение: `let memory = ask::variant("Память", args.memory)?;` и передача `memory` в `Ok(Action::Start { ... })` |
| 7 | `src/cli.rs` | Тесты аргументов | Проверка опциональности в `argument_tests::every_start_argument_is_optional` |
| 8 | `src/cli.rs` | Тесты заполнения | Обновление паттернов и передача аргументов в `fill_tests::full_start_arguments_need_no_questions` и `start_accepts_a_markdown_prompt` |

---

## Поведение компилятора при пропуске шагов

### 1. Пропущено объявление `enum Memory` в `src/lib.rs` (но поле указано в `Action` и `cli.rs`)
- Команда: `cargo check --all-targets`
- Код ошибки: `E0425`, `E0432`
- Первая строка: `error[E0432]: unresolved import crate::Memory` / `error[E0425]: cannot find type Memory in this scope`
- Имя поля видно: **нет** (в сообщении фигурирует имя типа `Memory`).

### 2. Пропущено поле в целевом типе `Action::Start` в `src/lib.rs` (но `Memory`, `StartArgs` и `fn start` обновлены)
- Команда: `cargo check --all-targets`
- Код ошибки: `E0559`
- Первая строка: `error[E0559]: variant Action::Start has no field named memory`
- Имя поля видно: **да** (`memory`).

### 3. Пропущено поле `memory` в `StartArgs` в `src/cli.rs` (но `Memory`, `Action::Start` и `fn start` обновлены)
- Команда: `cargo check --all-targets`
- Код ошибки: `E0609`
- Первая строка: `error[E0609]: no field memory on type StartArgs`
- Имя поля видно: **да** (`memory`).

### 4. Пропущено заполнение/инициализация в `fn start` в `src/cli.rs` (но `Memory`, `Action::Start` и `StartArgs` обновлены)
- Команда: `cargo check --all-targets`
- Код ошибки: `E0063`
- Первая строка: `error[E0063]: missing field memory in initializer of Action`
- Имя поля видно: **да** (`memory`).

### 5. Обновлён `Action::Start` в `src/lib.rs`, но `src/cli.rs` не тронут вовсе
- Команда: `cargo check --all-targets`
- Код ошибки: `E0063`, `E0027`
- Первая строка: `error[E0063]: missing field memory in initializer of Action`
- Имя поля видно: **да** (`memory`).

### 6. Добавлен флаг в `StartArgs` в `src/cli.rs`, но пропущены `Action::Start` и `fn start`
- Команда: `cargo check --all-targets`
- Код ошибки: **ошибок нет** (код возврата `0`)
- Первая строка: сборка завершается успешно (`Finished dev profile`).
- Имя поля видно: **нет**.
- Наблюдаемое поведение: CLI принимает `--memory off` / `--memory shared`, но значение тихо отбрасывается и не попадает в команду. Требование 7 нарушается при ручной зеркализации `StartArgs` и `Action::Start`.

---

## Дифф добавления параметра

```diff
diff --git a/src/cli.rs b/src/cli.rs
index e4261da..eb1e4eb 100644
--- a/src/cli.rs
+++ b/src/cli.rs
@@ -6,7 +6,7 @@ use clap_complete::CompleteEnv;
 use clap_complete::engine::{ArgValueCandidates, ArgValueCompleter, PathCompleter};
 
 use crate::ask;
-use crate::{Action, Agent, Base, Command, Prompt, complete};
+use crate::{Action, Agent, Base, Command, Memory, Prompt, complete};
 
 #[derive(Parser, Debug)]
 #[command(name = "cat-context", version)]
@@ -46,6 +46,9 @@ struct StartArgs {
     #[arg(long, value_enum, value_name = "AGENT")]
     agent: Option<Agent>,
 
+    #[arg(long, value_enum, value_name = "MEMORY")]
+    memory: Option<Memory>,
+
     #[command(flatten)]
     prompt: PromptArgs,
 }
@@ -120,6 +123,7 @@ mod argument_tests {
 
         assert!(args.base.is_none());
         assert!(args.agent.is_none());
+        assert!(args.memory.is_none());
         assert!(args.prompt.file.is_none());
         assert!(args.prompt.text.is_none());
         assert!(!args.prompt.no_prompt);
@@ -348,6 +352,8 @@ mod fill_tests {
             "alpine",
             "--agent",
             "codex",
+            "--memory",
+            "off",
             "--text",
             "привет",
         ])
@@ -356,12 +362,14 @@ mod fill_tests {
         let Action::Start {
             base,
             agent,
+            memory,
             prompt,
         } = action
         else {
             panic!("ожидался Start");
         };
 
         assert_eq!(base, Base::Alpine);
         assert_eq!(agent, Agent::Codex);
+        assert_eq!(memory, Memory::Off);
         assert!(matches!(prompt, Prompt::Text(text) if text == "привет"));
@@ -376,6 +384,8 @@ mod fill_tests {
             "arch",
             "--agent",
             "opencode",
+            "--memory",
+            "shared",
             "--file",
             "plan.md",
         ])
@@ -525,6 +535,7 @@ mod is_visitable_tests {
 fn start(args: StartArgs) -> ask::Result<Action> {
     let base = ask::variant("Базовый образ", args.base)?;
     let agent = ask::variant("Агент", args.agent)?;
+    let memory = ask::variant("Память", args.memory)?;
 
     let prompt = match args.prompt.into_prompt() {
         Some(prompt) => prompt,
@@ -534,6 +545,7 @@ fn start(args: StartArgs) -> ask::Result<Action> {
     Ok(Action::Start {
         base,
         agent,
+        memory,
         prompt,
     })
 }
diff --git a/src/lib.rs b/src/lib.rs
index b34d0b1..431ad29 100644
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -26,6 +26,13 @@ pub enum Agent {
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
@@ -39,6 +46,7 @@ pub enum Action {
     Start {
         base: Base,
         agent: Agent,
+        memory: Memory,
         prompt: Prompt,
     },
     Restart {
@@ -74,6 +82,13 @@ impl fmt::Display for Agent {
     }
 }
 
+impl fmt::Display for Memory {
+    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
+        let variant = self.to_possible_value().expect("вариант не скрыт");
+        f.write_str(variant.get_name())
+    }
+}
+
 pub async fn run() -> u8 {
     exit_code(cli::command().await)
 }
```
