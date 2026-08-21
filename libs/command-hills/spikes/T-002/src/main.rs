use clap::{Args, Parser};

// --- Target Types (from application logic) ---
pub struct Context; // Example external context

pub enum Action {
    Restart(RestartAction),
    Start(StartAction),
}

pub struct RestartAction {
    pub base: Option<String>,
    pub container: String,
    pub prompt: Prompt,
}

pub struct StartAction {
    pub container: String,
}

pub enum Prompt {
    File(String),
    Text(String),
    NoPrompt,
}

// =========================================================================
// DESIRED DECLARATION (What the user writes)
// =========================================================================

/*
#[command_hill(target = RestartAction)]
struct RestartCommand {
    // 1. Поле остаётся Option
    base: Option<String>,

    // 2. Поле только в аргументах (skip)
    #[hill(skip)]
    no_save: bool,

    #[hill(skip)]
    connect: bool,

    // 3. Поле меняет имя и форму (target: TargetArgs -> container: String)
    #[hill(rename_target = "container", resolve = "|args: &Self, ctx: &Context| args.target.resolve(ctx)")]
    #[command(flatten)]
    target: TargetArgs,

    // 4. Группа взаимоисключающих флагов сворачивается в enum
    #[hill(rename_target = "prompt", resolve = "|args: &Self, _| Prompt::from(args)")]
    #[command(flatten)]
    prompt_args: PromptArgs,

    // 5. Поле со своим резолвером и внешним контекстом
    // (This is covered by `resolve` attribute taking a closure with context)
}
*/

// =========================================================================
// MANUAL EXPANSION (What the macro generates)
// =========================================================================

#[derive(Args, Debug)]
pub struct TargetArgs {
    #[arg(long)]
    pub container_name: Option<String>,
}

impl TargetArgs {
    pub fn resolve(&self, _ctx: &Context) -> String {
        self.container_name
            .clone()
            .unwrap_or_else(|| "default".to_string())
    }
}

#[derive(Args, Debug)]
pub struct PromptArgs {
    #[arg(long)]
    pub file: Option<String>,
    #[arg(long)]
    pub text: Option<String>,
    #[arg(long)]
    pub no_prompt: bool,
}

impl PromptArgs {
    pub fn resolve(&self) -> Prompt {
        if let Some(f) = &self.file {
            Prompt::File(f.clone())
        } else if let Some(t) = &self.text {
            Prompt::Text(t.clone())
        } else {
            Prompt::NoPrompt
        }
    }
}

#[derive(Parser, Debug)]
pub struct RestartCommandArgs {
    #[arg(long)]
    pub base: Option<String>,

    #[arg(long)]
    pub no_save: bool,

    #[arg(long)]
    pub connect: bool,

    #[command(flatten)]
    pub target: TargetArgs,

    #[command(flatten)]
    pub prompt_args: PromptArgs,
}

impl RestartCommandArgs {
    pub fn resolve(&self, ctx: &Context) -> RestartAction {
        RestartAction {
            base: self.base.clone(), // 1. Option remains Option
            // 2. no_save and connect are skipped, so they are just ignored here
            container: self.target.resolve(ctx), // 3 and 5. rename and external context
            prompt: self.prompt_args.resolve(),  // 4. map flattening into enum
        }
    }
}

fn main() {
    println!("Compiles!");
}
