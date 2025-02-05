
# Custom web framework
This is a simple self made web framework made in Rust. Usefull to create an API.




## Features

- POST and GET request handling
- Parsing of request body in hashmap
- Plain text http response


## Startup

To start this project run

```bash
  cargo run
```

Use Postman to test your API


## How to use ?

```rust
use app::App;
use http_request::{HttpRequest, HttpResponse};


mod http_request;
mod app;

async fn home(req: HttpRequest, mut res: HttpResponse) {
    res.status(201).send("home").await;
}

async fn add(req: HttpRequest, mut res: HttpResponse) {
    res.status(201).send("add").await;
}

#[tokio::main]
async fn main() {
    let mut app = App::new("127.0.0.1:8080");

    app.get("/", home).await;
    app.post("/add", add).await;

    app.listen().await;
}

```


## Dependencies

Tokio and futures crate are needed

```bash
  cargo add tokio
  cargo add futures
```

Look at the repo cargo.toml for more details
    
## 🔗 Links
https://github.com/BriannGerbaux/Custom-Web-Framework.git
