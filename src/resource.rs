//! Definitions for resources and related types.
use std::any;
use std::collections;
use std::fmt;
use std::path;

use slog;

use error;

/// Something that is managed by `realize` and that can be realized on the
/// target system.
pub trait Resource: fmt::Debug + fmt::Display {
    /// The key of this resource, that distinguishes it from other resources of
    /// the same type.  For files, this might be the file path, for example.
    fn key(&self) -> Key;

    /// Realizes this resource, performing any needed operations on the target
    /// system.
    fn realize(&self, ctx: &Context) -> error::Result<()>;

    /// Verifies whether this resource is already realized on the target system.
    /// This will potentially perform a lot of IO operations but should not make
    /// any changes.
    fn verify(&self, ctx: &Context) -> error::Result<bool>;

    /// Converts this resource into an opaque `Any` reference.
    fn as_any(&self) -> &any::Any;
}

/// A resource key, used to uniquely disambiguate resources of the same type.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Key {
    /// A composite key consisting of several fields.
    Map(collections::BTreeMap<String, Key>),
    /// A composite key consisting of several ordered keys.
    Seq(Vec<Key>),
    /// A key that is based on a string value.
    String(String),
    /// A key that is based on a file system path.
    Path(path::PathBuf),
    // Add more stuff as needed
}

/// A `Resource` that is not yet "resolved," meaning that it might have
/// dependency resources that should be implicitly realized as well.
pub trait UnresolvedResource: Resource + Eq {
    /// Ensures that any implicit resources that this resource depends on are
    /// realized.
    #[allow(unused_variables)]
    fn implicit_ensure<E>(&self, ensurer: &mut E)
    where
        E: Ensurer,
    {
    }
}

/// Something that can ensure that resources are realized.
pub trait Ensurer {
    /// Ensures that the specified resource is realized, including its
    /// unresolved dependencies.
    fn ensure<R>(&mut self, resource: R)
    where
        R: UnresolvedResource + 'static;
}

/// A context that is passed around various parts of the library, containing
/// common functionality.
#[derive(Debug)]
pub struct Context<'a> {
    /// A structured logger to use when logging contextual information.
    pub log: &'a slog::Logger,
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Key::Map(ref value) => {
                write!(f, "{{")?;
                let mut needs_sep = false;
                for (key, value) in value {
                    if needs_sep {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", key, value)?;
                    needs_sep = true;
                }
                write!(f, "}}")?;
            }
            Key::Seq(ref value) => {
                write!(f, "[")?;
                let mut needs_sep = false;
                for value in value {
                    if needs_sep {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", value)?;
                    needs_sep = true;
                }
                write!(f, "]")?;
            }
            Key::String(ref value) => write!(f, "{:?}", value)?,
            Key::Path(ref path) => write!(f, "{:?}", path)?,
        }
        Ok(())
    }
}
