//! A blazingly fast configuration management library. Exposes a type-safe eDSL
//! for writing system configuration programs.
//!
//! # Terminology
//!
//! This library realizes configurations. A configuration is a collection of
//! `Resource`s, that are declared using an eDSL. When all resources have been
//! realized, the configuration is considered applied.
//!
//! To aid with dependency management, there are meta-resources that can manage
//! several resources and the dependencies between them. One such resource is
//! the `Reality` resource, that can `ensure` that other resources are realized.

#![deny(
    missing_docs, missing_debug_implementations, missing_copy_implementations, trivial_casts,
    trivial_numeric_casts, unsafe_code, unstable_features, unused_import_braces,
    unused_qualifications
)]

extern crate failure;
extern crate linked_hash_map;
extern crate sha1;
#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_journald;
extern crate slog_term;

pub mod error;
pub mod resource;
mod util;

pub mod fs;
pub mod meta;

pub use meta::Reality;

use std::process;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

/// Runs the supplied configuration function and applies the resulting reality.
/// This should be called from your `main` method, it assumes that it is safe to
/// call `std::process::exit` for example.
pub fn apply<F>(config: F)
where
    F: FnOnce(&mut Reality),
{
    use slog::Drain;
    let decorator = slog_term::TermDecorator::new().stderr().build();
    let term_drain = slog_term::CompactFormat::new(decorator)
        .use_utc_timestamp()
        .build()
        .fuse();
    let term_drain = slog_async::Async::new(term_drain).build().fuse();
    let journald_drain = slog_journald::JournaldDrain.fuse();

    let drain = slog::Duplicate::new(term_drain, journald_drain).fuse();
    let log = slog::Logger::root(drain.fuse(), o!("realize-version" => VERSION));

    match apply_inner(&log, config) {
        Ok(()) => (),
        Err(e) => {
            error!(log, "{}", e);
            for error in e.iter_causes() {
                error!(log, " → {}", error);
            }
            // Ensure we log everything
            drop(log);
            process::exit(1);
        }
    }
}

fn apply_inner<F>(log: &slog::Logger, config: F) -> error::Result<()>
where
    F: FnOnce(&mut Reality),
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
