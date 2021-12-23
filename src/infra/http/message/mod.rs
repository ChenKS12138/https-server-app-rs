mod fsm;

use rust_fsm::StateMachine;
use std::{
    cell::RefCell,
    collections::HashMap,
    io::{Read, Write},
    rc::Rc,
    sync::Arc,
};

pub type HandleFn = Box<Arc<dyn Fn(Rc<RefCell<Request>>) -> Response + Send + Sync>>;

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
    pub fn new() -> Self {
        Self {
            version: String::from(HTTP_VERSION),
            body: Vec::new(),
            code: super::status::OK,
            headers: HashMap::new(),
        }
    }
    pub fn with_text(code: super::status::Status, text: &str) -> Response {
        let mut response = Response::new();
        response.code = code;
        response.set_body(&Vec::from(text));
        response
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

struct Parser<'a, T: Read> {
    readable: &'a mut T,
    machine: StateMachine<fsm::RequestMessage>,
}

impl<'a, T: Read> Parser<'a, T> {
    fn new(readable: &mut T) -> Parser<T> {
        Parser {
            readable,
            machine: StateMachine::new(),
        }
    }
    fn parse(&mut self) -> Result<Option<Request>, Box<dyn std::error::Error>> {
        let mut body: Vec<u8> = Vec::new();
        let mut headers: HashMap<String, String> = HashMap::new();
        let mut header_field = String::new();
        let mut header_value = String::new();
        let mut method = String::new();
        let mut path = String::new();
        let mut version = String::new();
        let mut rest_body_size = 0_u64;

        let mut byte = [0_u8; 1];
        loop {
            if let Err(_) = self.readable.read_exact(&mut byte) {
                return Ok(None);
            };
            let byte = byte[0];

            let effect = if rest_body_size == 0
                && (self.machine.state() == &fsm::RequestMessageState::Lf2
                    || self.machine.state() == &fsm::RequestMessageState::Body)
            {
                self.machine.consume(&fsm::RequestMessageInput::End)
            } else {
                match byte {
                    b' ' => self.machine.consume(&fsm::RequestMessageInput::Blank),
                    b':' => self.machine.consume(&fsm::RequestMessageInput::Colon),
                    b'\r' => self.machine.consume(&fsm::RequestMessageInput::Cr),
                    b'\n' => self.machine.consume(&fsm::RequestMessageInput::Lf),
                    _ => self.machine.consume(&fsm::RequestMessageInput::Alpha),
                }
            };

            match effect? {
                Some(effect) => match effect {
                    fsm::RequestMessageOutput::EffectAppendHeader => {
                        headers.insert(header_field.clone(), header_value.clone());
                        header_field.clear();
                        header_value.clear();
                    }
                    fsm::RequestMessageOutput::EffectAppendHeaderField => {
                        header_field.push(char::from(byte));
                    }
                    fsm::RequestMessageOutput::EffectAppendHeaderValue => {
                        header_value.push(char::from(byte));
                    }
                    fsm::RequestMessageOutput::EffectAppendMethod => {
                        method.push(char::from(byte));
                    }
                    fsm::RequestMessageOutput::EffectAppendPath => {
                        path.push(char::from(byte));
                    }
                    fsm::RequestMessageOutput::EffectAppendVersion => {
                        version.push(char::from(byte));
                    }
                    _ => {
                        match effect {
                            fsm::RequestMessageOutput::EffectCheckEnd => {
                                if let Some(content_length) = headers.get("Content-Length") {
                                    rest_body_size = content_length.parse().unwrap_or(0);
                                }
                            }
                            fsm::RequestMessageOutput::EffectAppendBody => {
                                body.push(byte);
                                rest_body_size -= 1;
                            }
                            _ => {}
                        }
                        if rest_body_size == 0 {
                            let request = super::message::Request {
                                body,
                                headers,
                                method,
                                path,
                                version,
                            };
                            self.machine.consume(&fsm::RequestMessageInput::End)?;
                            return Ok(Some(request));
                        }
                    }
                },
                _ => {
                    // do nothing
                }
            }
        }
    }
}

pub fn consume<T: Write + Read>(
    connection: &mut T,
    on_data: HandleFn,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut parser = Parser::new(connection);
    if let Some(request) = parser.parse().unwrap() {
        let response = on_data(Rc::new(RefCell::new(request)));
        connection.write_all(&response.to_bytes().unwrap())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        cmp,
        io::{self, Write},
    };

    use crate::infra::http::{
        message::{HttpMessage, Parser},
        method::{get_methods, Method},
    };
    struct StringStream {
        data: Vec<u8>,
        index: usize,
    }
    impl StringStream {
        fn new(data: &'static str) -> Self {
            Self {
                data: Vec::from(data),
                index: 0,
            }
        }
    }

    impl io::Read for StringStream {
        fn read(&mut self, mut buf: &mut [u8]) -> io::Result<usize> {
            let size = cmp::min(buf.len(), self.data.len() - self.index);
            let result = buf.write(&self.data[self.index..(self.index + size)]);
            self.index += size;
            result
        }
    }

    #[test]
    fn parse_request_get() {
        let mut readable = StringStream::new("GET / HTTP/1.1\r\nHost: 127.0.0.1:3000\r\nUser-Agent: curl/7.64.1\r\nAccept: */*\r\n\r\n");
        let mut parser = Parser::new(&mut readable);
        let request = parser.parse().unwrap().unwrap();
        assert_eq!(get_methods(request.method.as_str()).unwrap(), Method::Get);
        assert_eq!(request.path, "/");
        assert_eq!(request.get_header("Host").unwrap(), "127.0.0.1:3000");
        assert_eq!(request.get_header("User-Agent").unwrap(), "curl/7.64.1");
        assert_eq!(request.get_header("Accept").unwrap(), "*/*");
    }
    #[test]
    fn parse_request_post() {
        let mut readable = StringStream::new("POST /user HTTP/1.1\r\nHost: 127.0.0.1:3000\r\nUser-Agent: curl/7.64.1\r\nAccept: */*\r\nContent-Type: application/json\r\nContent-Length: 23\r\n\r\n{\"name\":\"tom\",\"age\":21}");
        let mut parser = Parser::new(&mut readable);
        let request = parser.parse().unwrap().unwrap();
        assert_eq!(get_methods(request.method.as_str()).unwrap(), Method::Post);
        assert_eq!(request.path, "/user");
        assert_eq!(request.get_header("Host").unwrap(), "127.0.0.1:3000");
        assert_eq!(request.get_header("User-Agent").unwrap(), "curl/7.64.1");
        assert_eq!(request.get_header("Accept").unwrap(), "*/*");
        assert_eq!(
            request.get_header("Content-Type").unwrap(),
            "application/json"
        );
        assert_eq!(request.get_header("Content-Length").unwrap(), "23");
        assert_eq!(request.body, Vec::from("{\"name\":\"tom\",\"age\":21}"));
    }
}
