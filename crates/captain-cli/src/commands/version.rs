//! `captain version`.

pub fn run() {
    println!("captain {}", env!("CARGO_PKG_VERSION"));
}
