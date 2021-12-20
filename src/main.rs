#[macro_use]
extern crate lazy_static;

mod app;
mod infra;
mod ui;

fn main() {
    app::serve(
        String::from("0.0.0.0:443"),
        String::from("/Users/cattchen/Codes/https-server-app/script/server_bundle.cert.pem"),
        String::from("/Users/cattchen/Codes/https-server-app/script/server.private.pem"),
    )
    .unwrap();
}
