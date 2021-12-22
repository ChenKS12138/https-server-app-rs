use openssl::ssl::SslStream;
use rust_fsm::StateMachine;
use std::{
    collections::HashMap,
    io::{Read, Write},
    sync::Arc,
};
pub type HandleFn = Box<Arc<dyn Fn(Request) -> Response + Send + Sync>>;

const HTTP_VERSION: &str = "1.1";

pub trait HttpMessage {
    fn get_header(&self, key: &str) -> Option<&str>;
    fn set_header(&mut self, key: &str, value: &str);
}

#[derive(Debug, Clone)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl Request {}

impl HttpMessage for Request {
    fn get_header(&self, key: &str) -> Option<&str> {
        Some(self.headers.get(key)?.as_str())
    }
    fn set_header(&mut self, key: &str, value: &str) {
        self.headers.insert(String::from(key), String::from(value));
    }
}

#[derive(Debug, Clone)]
pub struct Response {
    pub version: String,
    pub code: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}
impl Response {
    pub fn new() -> Response {
        Response {
            version: String::from(HTTP_VERSION),
            body: Vec::new(),
            code: super::status::OK,
            headers: HashMap::new(),
        }
    }
    pub fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        use std::fmt::Write;
        let mut bytes_msg = String::new();
        bytes_msg.write_fmt(format_args!(
            "HTTP/{} {} {}\r\n",
            self.version,
            self.code,
            super::status::get_code_reason(self.code).ok_or("unknown")?
        ))?;
        for (key, value) in &self.headers {
            bytes_msg.write_fmt(format_args!("{}: {}\r\n", key, value))?;
        }
        bytes_msg.write_fmt(format_args!("\r\n"))?;
        let mut body = self.body.clone();
        unsafe {
            bytes_msg.as_mut_vec().append(&mut body);
        }
        Ok(bytes_msg.as_bytes().to_vec())
    }
    pub fn set_body(&mut self, body: &Vec<u8>) {
        self.body = body.clone();
    }
    pub fn set_code(&mut self, code: u16) {
        self.code = code;
    }
}

impl HttpMessage for Response {
    fn get_header(&self, key: &str) -> Option<&str> {
        Some(self.headers.get(key)?.as_str())
    }
    fn set_header(&mut self, key: &str, value: &str) {
        self.headers.insert(String::from(key), String::from(value));
    }
}

pub fn consume<T: Write + Read>(
    mut connection: SslStream<T>,
    on_data: HandleFn,
) -> Result<(), Box<dyn std::error::Error>> {
    use super::fsm;

    let mut machine: StateMachine<fsm::RequestParser> = StateMachine::new();
    let mut body: Vec<u8> = Vec::new();
    let mut headers: HashMap<String, String> = HashMap::new();
    let mut header_field = String::new();
    let mut header_value = String::new();
    let mut method = String::new();
    let mut path = String::new();
    let mut version = String::new();
    let rest_body_size = 0_u64;

    let mut byte = [0_u8; 1];
    loop {
        if let Err(_) = connection.read_exact(&mut byte) {
            return Ok(());
        };
        let byte = byte[0];

        let effect = if rest_body_size == 0
            && (machine.state() == &fsm::RequestParserState::Lf2
                || machine.state() == &fsm::RequestParserState::Body)
        {
            machine.consume(&fsm::RequestParserInput::End)
        } else {
            match byte {
                b' ' => machine.consume(&fsm::RequestParserInput::Blank),
                b':' => machine.consume(&fsm::RequestParserInput::Colon),
                b'\r' => machine.consume(&fsm::RequestParserInput::Cr),
                b'\n' => machine.consume(&fsm::RequestParserInput::Lf),
                _ => machine.consume(&fsm::RequestParserInput::Alpha),
            }
        };

        match effect? {
            Some(effect) => match effect {
                fsm::RequestParserOutput::EffectAppendHeader => {
                    headers.insert(header_field.clone(), header_value.clone());
                }
                fsm::RequestParserOutput::EffectAppendHeaderField => {
                    header_field.push(char::from(byte));
                }
                fsm::RequestParserOutput::EffectAppendHeaderValue => {
                    header_value.push(char::from(byte));
                }
                fsm::RequestParserOutput::EffectAppendMethod => {
                    method.push(char::from(byte));
                }
                fsm::RequestParserOutput::EffectAppendPath => {
                    path.push(char::from(byte));
                }
                fsm::RequestParserOutput::EffectAppendVersion => {
                    version.push(char::from(byte));
                }
                _ => {
                    if effect == fsm::RequestParserOutput::EffectAppendBody {
                        body.push(byte);
                    }
                    if rest_body_size == 0 {
                        let request = super::message::Request {
                            body: body.clone(),
                            headers: headers.clone(),
                            method: method.clone(),
                            path: path.clone(),
                            version: version.clone(),
                        };
                        body = Vec::new();
                        headers = HashMap::new();
                        method = String::new();
                        path = String::new();
                        version = String::new();
                        let response = on_data(request);
                        connection.write_all(&(response.to_bytes()?))?;
                        connection.flush()?;
                        machine.consume(&fsm::RequestParserInput::End)?;
                        connection.shutdown()?;
                    }
                }
            },
            _ => {
                // do nothing
            }
        }
    }
}

#[cfg(test)]
mod tests {}
// sec-ch-ua
