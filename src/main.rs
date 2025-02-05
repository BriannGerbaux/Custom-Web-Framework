use app::App;
use http_request::{HttpRequest, HttpResponse};


mod http_request;
mod app;

async fn home(req: HttpRequest, mut res: HttpResponse) {
    res.status(201).send("test").await;
}

#[tokio::main]
async fn main() {
    let mut app = App::new("127.0.0.1:8080");

    app.get("/", home).await;

    app.listen().await;
}
