use app::App;
use http_request::{HttpRequest, HttpResponse};


mod http_request;
mod app;

async fn home(_req: HttpRequest, mut res: HttpResponse) {
    res.status(201).send("Home").await;
}

async fn add(req: HttpRequest, mut res: HttpResponse) {
    let body_json = match req.body {
        http_request::RequestBody::String(_) => return res.status(400).send("Wrong body form").await,
        http_request::RequestBody::Map(map) => map,
    };

    let value_option = body_json.get("value");

    let Some(value) = value_option else { return res.status(400).send("No value in body").await; };
    let Some(value_as_i64) = value.as_i64() else { return res.status(400).send("Wrong value").await; };

    if value_as_i64 < 30 {
        res.status(200).send("Value is lesser than 30").await;
    } else {
        res.status(200).send("Value is equal or greater than 30").await;
    }
}

#[tokio::main]
async fn main() {
    let mut app = App::new("127.0.0.1:8080");

    app.get("/", home).await;
    app.post("/add", add).await;

    app.listen().await;
}
