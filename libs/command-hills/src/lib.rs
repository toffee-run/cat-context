pub use command_hills_macros::{commands, fill, group, root};

pub type Result<T> = std::result::Result<T, inquire::InquireError>;

pub trait Resolve<Target> {
    fn resolve(self) -> Result<Target>;
}

pub trait ResolveWithCtx<Ctx, Target> {
    fn resolve(self, ctx: &Ctx) -> impl Future<Output = Result<Target>>;
}

#[doc(hidden)]
pub mod __private {
    pub use crate::question::{ask_subcommand, ask_variant, ask_variant_or_keep};
    pub use clap_complete::engine::{ArgValueCandidates, ArgValueCompleter, CompletionCandidate};

    pub fn resolve_field<Target>(value: impl crate::Resolve<Target>) -> crate::Result<Target> {
        crate::Resolve::resolve(value)
    }

    pub async fn resolve_field_with_ctx<Ctx, Target>(
        value: impl crate::ResolveWithCtx<Ctx, Target>,
        ctx: &Ctx,
    ) -> crate::Result<Target> {
        crate::ResolveWithCtx::resolve(value, ctx).await
    }

    pub fn clap_result<T, E>(result: std::result::Result<T, E>) -> crate::Result<T>
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        result.map_err(|error| inquire::InquireError::Custom(Box::new(error)))
    }

    pub fn required_subcommand<T>(value: Option<T>) -> crate::Result<T> {
        value.ok_or_else(|| {
            inquire::InquireError::InvalidConfiguration("подкоманда не выбрана".to_owned())
        })
    }
}

mod question;
