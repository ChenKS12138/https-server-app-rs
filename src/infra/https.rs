use crate::infra;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use std::{
    net::TcpListener,
    sync::mpsc::{channel, SendError, Sender},
};
use threadpool::ThreadPool;

use crate::infra::http::message::HandleFn;

pub struct HttpsServer {
    bind_addr: String,
    cert: String,
    key: String,
    tx: Option<Sender<()>>,
}

impl HttpsServer {
    pub fn new(bind_addr: String, cert: String, key: String) -> HttpsServer {
        HttpsServer {
            bind_addr,
            cert,
            key,
            tx: None,
        }
    }
    pub fn launch(&mut self, on_request: HandleFn) -> Result<(), Box<dyn std::error::Error>> {
        assert!(self.tx.is_none());
        let (tx, rx) = channel();
        let (bind_addr, cert, key) = (self.bind_addr.clone(), self.cert.clone(), self.key.clone());
        std::thread::spawn(move || {
            let pool = ThreadPool::new(num_cpus::get());
            let mut acceptor = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
            acceptor
                .set_private_key_file(key, SslFiletype::PEM)
                .unwrap();
            acceptor.set_certificate_chain_file(cert).unwrap();

            let listener = TcpListener::bind(bind_addr).unwrap();
            let acceptor = acceptor.build();

            for connection in listener.incoming() {
                if rx.try_recv().is_ok() {
                    return;
                }
                let connection = acceptor.accept(connection.unwrap());
                let on_request = on_request.clone();
                pool.execute(move || {
                    if connection.is_err() {
                        eprintln!("{:?}", connection.err());
                        return;
                    }
                    let mut connection = connection.unwrap();
                    infra::http::message::consume(&mut connection, on_request).unwrap();
                    connection.shutdown().unwrap();
                })
            }
        });
        self.tx = Some(tx);
        Ok(())
    }
    pub fn shutdown(&mut self) -> Result<(), SendError<()>> {
        let result = self.tx.as_ref().unwrap().send(());
        self.tx = None;
        result
    }
}
