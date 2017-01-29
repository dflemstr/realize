extern crate realize;

use realize::fs;

fn main() {
    realize::apply(configuration)
}

fn configuration(reality: &mut realize::Reality) {
    reality.ensure(fs::File::at("/tmp/test").contains_str("hello"));
}
