use std::any;
use std::collections;
use std::fmt;
use std::path;

use slog;

use error;

pub trait Resource: fmt::Debug + fmt::Display {
    fn key(&self) -> Key;

    fn realize(&self, ctx: &Context) -> error::Result<()>;

    fn verify(&self, ctx: &Context) -> error::Result<bool>;

    fn as_any(&self) -> &any::Any;
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Key {
    Map(collections::BTreeMap<String, Key>),
    Seq(Vec<Key>),
    String(String),
    Path(path::PathBuf),
    // Add more stuff as needed
}

pub trait UnresolvedResource: Resource + Eq {
    #[allow(unused_variables)]
    fn implicit_ensure<E>(&self, ensurer: &mut E)
    where
        E: Ensurer,
    {
    }
}

pub trait Ensurer {
    fn ensure<R>(&mut self, resource: R)
    where
        R: UnresolvedResource + 'static;
}

pub struct Context<'a> {
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
