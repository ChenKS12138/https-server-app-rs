use crate::infra::https::HttpsServer;

#[derive(Debug)]
pub struct AppState {
    pub server: HttpsServer,
    pub root_directory: Option<String>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            server: HttpsServer::new(),
            root_directory: None,
        }
    }
}
