pub mod fsm;
pub mod message;
pub mod method;
pub mod status;

#[derive(Debug)]
pub struct Error {
    message: &'static str,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Http Error {}", self.message)
    }
}

impl std::error::Error for Error {}
