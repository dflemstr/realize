use std::any;
use std::fmt;

use linked_hash_map;
use slog;

use error;
use resource;

#[derive(Debug)]
pub struct Reality {
    log: slog::Logger,
    resources: linked_hash_map::LinkedHashMap<(any::TypeId, resource::Key),
                                              Box<resource::Resource>>,
}

impl Reality {
    pub fn new(log: slog::Logger) -> Reality {
        Reality {
            log: log,
            resources: linked_hash_map::LinkedHashMap::new(),
        }
    }

    pub fn ensure<R>(&mut self, resource: R)
        where R: resource::UnresolvedResource + resource::Resource + 'static
    {
        resource.implicit_ensure(self);
        let key = (any::TypeId::of::<R>(), resource.key());
        // TODO: replace this with entry API once linked-hash-map supports that
        if self.resources.contains_key(&key) {
            let existing = self.resources[&key].as_any().downcast_ref::<R>().unwrap();
            if *existing != resource {
                let key = format!("{}", key.1);
                let old = format!("{}", existing);
                let new = format!("{}", resource);
                warn!(self.log, "Duplicate resource definitions; will use the older one";
                      "key" => key, "old" => old, "new" => new);
            }
        } else {
            self.resources.insert(key, Box::new(resource));
        }
    }
}


impl resource::Resource for Reality {
    fn key(&self) -> resource::Key {
        resource::Key::Seq(self.resources.values().map(|r| r.key()).collect())
    }

    fn realize(&self, ctx: &resource::Context) -> error::Result<()> {
        use error::ResultExt;
        for (_, resource) in &self.resources {
            resource.realize(ctx).chain_err(|| format!("Could not realize {}", resource))?;
        }

        Ok(())
    }

    fn verify(&self, ctx: &resource::Context) -> error::Result<bool> {
        use error::ResultExt;

        for (_, resource) in &self.resources {
            if !resource.verify(ctx).chain_err(|| format!("Could not verify {}", resource))? {
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
        where R: resource::UnresolvedResource + 'static
    {
        Reality::ensure(self, resource)
    }
}

impl fmt::Display for Reality {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "reality")
    }
}
