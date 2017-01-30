#[macro_use]
extern crate error_chain;
extern crate linked_hash_map;
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

use std::process;

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");

pub fn apply<F>(config: F)
    where F: FnOnce(&mut Reality),
{
    use slog::DrainExt;
    let term_drain = slog_term::streamer().async().stderr().use_utc_timestamp().compact().build();
    let journald_drain = slog_journald::JournaldDrain;

    let drain = slog::Duplicate::new(term_drain, journald_drain);
    let log = slog::Logger::root(drain.fuse(), o!("realize-version" => VERSION));

    match apply_inner(&log, config) {
        Ok(()) => (),
        Err(e) => {
            for error in e.iter() {
                error!(log, "{}", error);
            }
            // Ensure we log everything
            drop(log);
            process::exit(1);
        }
    }
}

pub fn apply_inner<F>(log: &slog::Logger, config: F) -> error::Result<()>
    where F: FnOnce(&mut Reality),
{
    use resource::Resource;

    let context = resource::Context { log: log };

    let mut resource = Reality::new(log.clone());

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
