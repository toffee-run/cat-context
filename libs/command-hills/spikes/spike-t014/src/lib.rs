pub mod part1 {
    use std::path::PathBuf;

    #[derive(Debug, PartialEq)]
    pub enum Prompt {
        File(PathBuf),
        Text(String),
        None,
    }

    #[derive(clap::Args, Default)]
    #[group(multiple = false)]
    pub struct PromptArgs {
        #[arg(long, value_name = "FILE")]
        pub file: Option<PathBuf>,
        #[arg(long, value_name = "TEXT")]
        pub text: Option<String>,
        #[arg(long)]
        pub no_prompt: bool,
    }

    impl command_hills::Resolve<Prompt> for PromptArgs {
        fn resolve(self) -> command_hills::Result<Prompt> {
            if let Some(file) = self.file {
                return Ok(Prompt::File(file));
            }
            if let Some(text) = self.text {
                return Ok(Prompt::Text(text));
            }
            if self.no_prompt {
                return Ok(Prompt::None);
            }
            // Cannot mechanically generate interactive prompt for data enum!
            // E.g. we don't know how to ask for a file or text.
            Err(inquire::InquireError::Custom("Нужно указать --file, --text или --no-prompt".into()))
        }
    }

    impl command_hills::Resolve<Option<Prompt>> for PromptArgs {
        fn resolve(self) -> command_hills::Result<Option<Prompt>> {
            if let Some(file) = self.file {
                return Ok(Some(Prompt::File(file)));
            }
            if let Some(text) = self.text {
                return Ok(Some(Prompt::Text(text)));
            }
            if self.no_prompt {
                return Ok(Some(Prompt::None));
            }
            Ok(None)
        }
    }
}
pub mod part2;
