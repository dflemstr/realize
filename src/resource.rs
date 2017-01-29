use std::fmt;

use slog;

use error;

pub trait Resource: fmt::Debug {
    fn realize(&self, ctx: &Context) -> error::Result<()>;

    fn verify(&self, ctx: &Context) -> error::Result<bool>;
}

pub trait UnresolvedResource: Resource {
    #[allow(unused_variables)]
    fn implicit_ensure<E>(&self, ensurer: &mut E) where E: Ensurer {}
}

pub trait Ensurer {
    fn ensure<R>(&mut self, resource: R) where R: UnresolvedResource + 'static;
}

pub struct Context<'a> {
    pub log: &'a slog::Logger,
}
