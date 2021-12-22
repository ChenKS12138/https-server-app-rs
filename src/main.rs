#[macro_use]
extern crate lazy_static;

mod app;
mod cmd;
mod infra;

fn main() {
    cmd::boost();
}
