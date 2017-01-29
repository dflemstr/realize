use error;
use resource;

#[derive(Debug, Default)]
pub struct Reality {
    resources: Vec<Box<resource::Resource>>,
}

impl Reality {
    pub fn new() -> Reality {
        Reality::default()
    }

    pub fn ensure<R>(&mut self, resource: R)
        where R: resource::UnresolvedResource + 'static
    {
        resource.implicit_ensure(self);
        self.resources.push(Box::new(resource));
    }
}


impl resource::Resource for Reality {
    fn realize(&self, ctx: &resource::Context) -> error::Result<()> {
        for resource in &self.resources {
            resource.realize(ctx)?;
        }

        Ok(())
    }

    fn verify(&self, ctx: &resource::Context) -> error::Result<bool> {
        for resource in &self.resources {
            if !resource.verify(ctx)? {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

impl resource::Ensurer for Reality {
    fn ensure<R>(&mut self, resource: R)
        where R: resource::UnresolvedResource + 'static
    {
        Reality::ensure(self, resource)
    }
}
