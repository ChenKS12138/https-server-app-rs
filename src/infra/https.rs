use crate::infra;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use std::{
    net::TcpListener,
    sync::mpsc::{channel, SendError, Sender},
};
use threadpool::ThreadPool;

use crate::infra::http::message::HandleFn;

#[derive(Debug)]
pub enum HttpsServerStatus {
    Stopped,
    Starting,
    Started,
    Stopping,
}

#[derive(Debug)]
pub struct HttpsServer {
    pub bind_addr: Option<String>,
    pub cert: Option<String>,
    pub key: Option<String>,
    pub status: HttpsServerStatus,
    tx: Option<Sender<()>>,
}

impl HttpsServer {
    pub fn new() -> Self {
        Self {
            bind_addr: None,
            cert: None,
            key: None,
            status: HttpsServerStatus::Stopped,
            tx: None,
        }
    }
    // launch
    pub fn launch(&mut self, on_request: HandleFn) -> Result<(), Box<dyn std::error::Error>> {
        assert!(self.tx.is_none());
        self.status = HttpsServerStatus::Starting;
        let (tx, rx) = channel();
        let (bind_addr, cert, key) = (
            self.bind_addr
                .clone()
                .ok_or(infra::http::Error::new("no bind_addr"))?,
            self.cert
                .clone()
                .ok_or(infra::http::Error::new("no bind_addr"))?,
            self.key
                .clone()
                .ok_or(infra::http::Error::new("no bind_addr"))?,
        );
        std::thread::spawn(move || {
            // 创建线程池
            let pool = ThreadPool::new(num_cpus::get());

            // 创建SSL层
            let mut acceptor = SslAcceptor::mozilla_intermediate(SslMethod::tls_server()).unwrap();
            acceptor
                .set_private_key_file(key, SslFiletype::PEM)
                .unwrap();
            acceptor.set_certificate_chain_file(cert).unwrap();
            let acceptor = acceptor.build();

            // 创建传输层 TCP Listener
            let listener = TcpListener::bind(bind_addr).unwrap();

            // 处理连接
            for connection in listener.incoming() {
                //检查服务器启动状态
                if rx.try_recv().is_ok() {
                    return;
                }

                // TCP数据先经SSL层处理再传递给http层
                let connection = acceptor.accept(connection.unwrap());

                let on_request = on_request.clone();
                pool.execute(move || {
                    if connection.is_err() {
                        eprintln!("{:?}", connection.err());
                        return;
                    }
                    let mut connection = connection.unwrap();

                    // 从TCP流中提取HTTP报文并交给on_request处理后返回
                    infra::http::message::consume(&mut connection, on_request).unwrap();

                    // 关闭连接
                    connection.shutdown().unwrap();
                })
            }
        });
        self.tx = Some(tx);
        self.status = HttpsServerStatus::Started;
        Ok(())
    }
    // shutdown
    pub fn shutdown(&mut self) -> Result<(), SendError<()>> {
        self.status = HttpsServerStatus::Stopping;
        let result = self.tx.as_ref().unwrap().send(());
        self.tx = None;
        self.status = HttpsServerStatus::Stopped;
        result
    }
}
