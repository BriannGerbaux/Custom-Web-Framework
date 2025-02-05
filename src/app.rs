use std::collections::HashMap;
use std::sync::Arc;
use futures::future::{BoxFuture, FutureExt};
use tokio::io::BufStream;
use tokio::io::AsyncBufReadExt;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

use crate::http_request::{HttpRequest, HttpResponse, RequestType};

type Endpoints = HashMap<String, Arc<dyn Fn(HttpRequest, HttpResponse) -> BoxFuture<'static, ()> + Send + Sync>>;

pub struct App {
    addr: String,
    get_endpoints: Arc<Mutex<Endpoints>>,
    post_endpoints: Arc<Mutex<Endpoints>>,
}

impl App {
    pub fn new(addr: &str) -> App {
        App{ addr: addr.to_string(), get_endpoints: Arc::new(Mutex::new(HashMap::new())), post_endpoints: Arc::new(Mutex::new(HashMap::new())) }
    }

    pub async fn get<F, Fut>(&mut self, route: &str, callback: F)
    where 
        F: Fn(HttpRequest, HttpResponse) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        self.get_endpoints.lock().await.insert(route.to_string(), Arc::new(move |x, y| callback(x, y).boxed()));
    }

    pub async  fn post<F, Fut>(&mut self, route: &str, callback: F)
    where 
        F: Fn(HttpRequest, HttpResponse) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        self.post_endpoints.lock().await.insert(route.to_string(), Arc::new(move |x, y| callback(x, y).boxed()));
    }

    pub async fn listen(&self) {
        let listener = TcpListener::bind(self.addr.as_str()).await.unwrap();
    
        loop {
            let get_endpoints_clone = Arc::clone(&self.get_endpoints);
            let post_endpoints_clone = Arc::clone(&self.post_endpoints);
            let (socket, _) = listener.accept().await.unwrap();
            tokio::spawn(async move {
                let mut req = HttpRequest::new();
                let buf_stream = Arc::new(Mutex::new(BufStream::new(socket)));
    
                // Parse first line
                let mut request_block_buf: Vec<u8> = Vec::new();
                buf_stream.lock().await.read_until(b'\n', &mut request_block_buf).await.unwrap();
                let request_block =
                    String::from_utf8(request_block_buf).expect("Our bytes should be valid utf8").trim_end().to_string();
                req.parse_request_line(&request_block);

                // Parse header
                loop {
                    let mut block: Vec<u8> = Vec::new();
                    buf_stream.lock().await.read_until(b'\n', &mut block).await.unwrap();
                    let line = String::from_utf8(block).expect("Our bytes should be valid utf8");
                    if line == "\r\n" || line == "" {
                        break;
                    }
                    req.parse_header(&line.trim_end());
                }

                //TODO: parse body

                let func = match req.request_type {
                    RequestType::GET => Some(get_endpoints_clone.lock().await.get(&req.route).unwrap().clone()),
                    RequestType::POST => Some(post_endpoints_clone.lock().await.get(&req.route).unwrap().clone()),
                    RequestType::UNDEFINED => None,
                };


                let res = HttpResponse::new(buf_stream.clone());

                if func.is_some() {
                    func.unwrap()(req, res).await;
                };
            });
        }
    
    }
}