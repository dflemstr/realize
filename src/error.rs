//! Error types and utilities.
use std::io;

error_chain! {
    foreign_links {
        IO(io::Error)
            /// An operating system IO error
            ;
    }
}
