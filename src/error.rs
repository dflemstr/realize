use std::io;

error_chain! {
    foreign_links {
        IO(io::Error);
    }
}
