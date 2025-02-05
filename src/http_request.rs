use std::{cell::RefCell, collections::HashMap, io::Write, pin::Pin, rc::Rc, sync::Arc};
use serde::{Serialize, Deserialize};
use serde_json::Value;
use tokio::{io::{AsyncWriteExt, BufStream}, net::TcpStream, sync::Mutex};

pub enum RequestType {
    GET,
    POST,
    UNDEFINED
}

pub enum RequestBody {
    String(String),
    Map(HashMap<String, Value>),
}

pub struct RequestHeader {
    pub content_type: String,
    pub user_agent: String,
    pub host: String,
    pub content_length: u64,
}

impl Default for RequestHeader {
    fn default() -> Self {
        RequestHeader {
            content_type: "".to_string(),
            user_agent: "".to_string(),
            host: "".to_string(),
            content_length: 0,
        }
    }
}

pub struct HttpRequest {
    pub route: String,
    pub request_type: RequestType,
    pub header: RequestHeader,
    pub body: RequestBody,
}

impl HttpRequest {
    pub fn new() -> Self {
        HttpRequest {
            route: "".to_string(),
            request_type: RequestType::UNDEFINED,
            header: RequestHeader::default(),
            body: RequestBody::String("".to_string()),
        }
    }

    pub fn parse_request_line(&mut self, line: &str) {
        let mut split = line.split(' ');

        match split.next() {
            Some("GET") => self.request_type = RequestType::GET,
            Some("POST") => self.request_type = RequestType::POST,
            Some(_) => self.request_type = RequestType::UNDEFINED,
            None => self.request_type = RequestType::UNDEFINED,
        }

        match split.next() {
            Some(s) => self.route = s.to_string(),
            None => self.route = "".to_string()
        }
    }

    pub fn parse_header(&mut self, line: &str) {
        let mut split = line.split(": ");

        match split.next() {
            Some("Content-Type") => self.header.content_type = split.next().unwrap().to_string(),
            Some("User-Agent") => self.header.user_agent = split.next().unwrap().to_string(),
            Some("Host") => self.header.host = split.next().unwrap().to_string(),
            Some("Content-Length") => self.header.content_length = split.next().unwrap().parse::<u64>().unwrap(),
            Some(_) => (),
            None => (),
        }
    }

    pub fn parse_body(&mut self, body: &str) {
        if self.header.content_type.starts_with("text/") {
            self.body = RequestBody::String(body.to_string());
        } else if self.header.content_type == "application/json" {
            let map: HashMap<String, Value> = serde_json::from_str(body).unwrap();
            self.body = RequestBody::Map(map);
        } else if self.header.content_type == "application/x-www-form-urlencoded" {
            let map: HashMap<String, Value> = serde_urlencoded::from_str(body).unwrap();
            self.body = RequestBody::Map(map);
        }
    }
}

pub struct HttpResponse {
    tcp_stream: Arc<Mutex<BufStream<TcpStream>>>,
    code: u32,
    message: String,
}

impl HttpResponse {
    pub fn new(stream: Arc<Mutex<BufStream<TcpStream>>>) -> Self {
        Self { tcp_stream: stream, code: 200, message: "".to_string() }
    }

    pub fn status(&mut self, status_code: u32) -> &mut Self {
        self.code = status_code;
        return self;
    }

    pub async fn send_not_found(&self) {
        let response_str = &format!("HTTP/1.1 404 Not Found\r\nContent-Type: text/plain\r\nContent-length: 0\r\n\r\n");

        match self.tcp_stream.lock().await.write_all(response_str.as_bytes()).await {
            Err(e) => eprintln!("{}", e),
            Ok(_) => println!("Successfully sent"),
        }
        self.tcp_stream.lock().await.flush().await.unwrap();
    }

    pub async fn send(&mut self, msg: &str) {
        self.message = msg.to_string();

        // Insert status line in response string
        let mut response_str = String::new();
        response_str = response_str + &format!("HTTP/1.1 {} OK\r\n", self.code);
        
        // Insert all header in response string
        let mut header_map: HashMap<&str, &str> = HashMap::new();
        let length = format!("{}", self.message.len());

        header_map.insert("Content-Type", "text/plain");
        header_map.insert("Content-Length", &length);

        response_str = response_str + &header_map.iter()
            .map(|(key, value)| format!("{}: {}", key, value))
            .collect::<Vec<String>>()
            .join("\r\n") + "\r\n\r\n";

        response_str = response_str + &self.message;

        match self.tcp_stream.lock().await.write_all(response_str.as_bytes()).await {
            Err(e) => eprintln!("{}", e),
            Ok(_) => println!("Successfully sent"),
        }
        self.tcp_stream.lock().await.flush().await.unwrap();
    }
}