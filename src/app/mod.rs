use crate::infra::http::message::{HttpMessage, Request, Response};

use super::infra;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use std::net::TcpListener;

pub fn serve(
    bind_addr: String,
    cert: String,
    key: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut acceptor = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    acceptor
        .set_private_key_file(key, SslFiletype::PEM)
        .unwrap();
    acceptor.set_certificate_chain_file(cert).unwrap();

    let listener = TcpListener::bind(bind_addr)?;
    let acceptor = acceptor.build();

    for connection in listener.incoming() {
        let connection = acceptor.accept(connection?);
        if connection.is_err() {
            eprintln!("{:?}", connection.err());
            continue;
        }
        let connection = connection.unwrap();
        infra::http::message::consume(connection, |request: Request| -> Response {
            println!("{}", request.path);
            let mut resp = Response::new();
            resp.set_body(&Vec::from("hello"));
            resp.set_header("Content-Type", "text/plain");
            resp
        })?;
    }
    Ok(())
}
