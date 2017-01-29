#[macro_use]
extern crate error_chain;
extern crate sha1;
#[macro_use]
extern crate slog;
extern crate slog_journald;
extern crate slog_term;

pub mod error;
pub mod resource;
pub mod util;

pub mod fs;
pub mod meta;

pub use meta::Reality;

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");

pub fn apply<F, R>(config: F)
    where F: FnOnce(&mut R),
          R: resource::Resource + Default
{
    apply_inner(config).unwrap()
}

pub fn apply_inner<F, R>(config: F) -> error::Result<()>
    where F: FnOnce(&mut R),
          R: resource::Resource + Default
{
    use slog::DrainExt;
    let term_drain = slog_term::streamer().async().stderr().use_utc_timestamp().compact().build();
    let journald_drain = slog_journald::JournaldDrain;

    let drain = slog::Duplicate::new(term_drain, journald_drain);
    let log = slog::Logger::root(drain.fuse(), o!("realize-version" => VERSION));
    let context = resource::Context { log: &log };

    let mut resource = R::default();

    config(&mut resource);

    info!(log, "Applying configuration");
    if resource.verify(&context)? {
        info!(log, "Everything up to date, nothing to do");
    } else {
        resource.realize(&context)?;
        info!(log, "Configuration applied");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
