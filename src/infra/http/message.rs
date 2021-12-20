use std::{collections::HashMap, fmt::Write, io};

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
}

impl HttpMessage for Response {
    fn get_header(&self, key: &str) -> Option<&str> {
        Some(self.headers.get(key)?.as_str())
    }
    fn set_header(&mut self, key: &str, value: &str) {
        self.headers.insert(String::from(key), String::from(value));
    }
}

pub fn consume<T: io::Write + io::Read>(
    mut connection: T,
    on_data: fn(Request) -> Response,
) -> Result<(), Box<dyn std::error::Error>> {
    let iter = connection.bytes();
    for byte in iter {
        let byte = byte?;
        //
    }
    Ok(())
}

#[cfg(test)]
mod tests {}
