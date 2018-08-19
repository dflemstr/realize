//! Meta-resources, that manage other resources.
use std::any;
use std::fmt;

use linked_hash_map;
use slog;

use error;
use resource;

/// A meta-resource that ensures that other resources are realized, in the
/// correct dependency order.
#[derive(Debug)]
pub struct Reality {
    log: slog::Logger,
    resources:
        linked_hash_map::LinkedHashMap<(any::TypeId, resource::Key), Box<resource::Resource>>,
}

impl Reality {
    /// Constructs a new reality that logs to the specified logger.
    pub fn new(log: slog::Logger) -> Reality {
        Reality {
            log: log,
            resources: linked_hash_map::LinkedHashMap::new(),
        }
    }

    /// Adds a resource to be realized (including its dependencies) when this
    /// reality is realized.
    pub fn ensure<R>(&mut self, resource: R)
    where
        R: resource::UnresolvedResource + resource::Resource + 'static,
    {
        resource.implicit_ensure(self);
        let key = (any::TypeId::of::<R>(), resource.key());

        match self.resources.entry(key) {
            linked_hash_map::Entry::Occupied(entry) => {
                // This unwrap() should never panic since we use TypeId::of::<R>() as part of the
                // key
                let existing = entry.get().as_any().downcast_ref::<R>().unwrap();
                if *existing != resource {
                    let key = format!("{}", entry.key().1);
                    let old = format!("{}", existing);
                    let new = format!("{}", resource);
                    warn!(self.log, "Duplicate resource definitions; will use the older one";
                          "key" => key, "old" => old, "new" => new);
                }
            }
            linked_hash_map::Entry::Vacant(entry) => {
                entry.insert(Box::new(resource));
            }
        }
    }
}

impl resource::Resource for Reality {
    fn key(&self) -> resource::Key {
        resource::Key::Seq(self.resources.values().map(|r| r.key()).collect())
    }

    fn realize(&self, ctx: &resource::Context) -> error::Result<()> {
        use failure::ResultExt;

        for (_, resource) in &self.resources {
            resource
                .realize(ctx)
                .with_context(|_| format!("Could not realize {}", resource))?;
        }

        Ok(())
    }

    fn verify(&self, ctx: &resource::Context) -> error::Result<bool> {
        use failure::ResultExt;

        for (_, resource) in &self.resources {
            if !resource
                .verify(ctx)
                .with_context(|_| format!("Could not verify {}", resource))?
            {
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn as_any(&self) -> &any::Any {
        self
    }
}

impl resource::Ensurer for Reality {
    fn ensure<R>(&mut self, resource: R)
    where
        R: resource::UnresolvedResource + 'static,
    {
        Reality::ensure(self, resource)
    }
}

impl fmt::Display for Reality {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "reality")
    }
}
