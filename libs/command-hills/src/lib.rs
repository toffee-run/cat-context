pub use command_hills_macros::fill;

pub type Result<T> = std::result::Result<T, inquire::InquireError>;

pub trait Resolve<Target> {
    fn resolve(self) -> Result<Target>;
}

pub trait ResolveWithCtx<Ctx, Target> {
    fn resolve(self, ctx: &Ctx) -> impl Future<Output = Result<Target>>;
}

#[doc(hidden)]
pub mod __private {
    pub use crate::question::{ask_variant, ask_variant_or_keep};
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
}

mod question;
