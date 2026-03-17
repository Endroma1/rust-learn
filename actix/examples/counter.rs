use std::sync::Mutex;

use actix_web::{App, HttpResponse, HttpServer, Responder, get, post, web};

#[derive(Default)]
struct Data {
    counter: Mutex<i32>,
}
impl Data {
    fn increment(&self) {
        let mut c = self.counter.lock().unwrap();
        *c += 1
    }
    fn decrement(&self) {
        let mut c = self.counter.lock().unwrap();
        *c -= 1
    }
    fn counter(&self) -> i32 {
        let c = self.counter.lock().unwrap();
        *c
    }
}
#[get("/counter")]
async fn counter(data: web::Data<Data>) -> String {
    data.counter().to_string()
}

#[post("/increment")]
async fn increment(data: web::Data<Data>) -> impl Responder {
    data.increment();
    HttpResponse::Ok()
}

#[post("/decrement")]
async fn decrement(data: web::Data<Data>) -> impl Responder {
    data.decrement();
    HttpResponse::Ok()
}
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .app_data(web::Data::new(Data::default()))
            .service(counter)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await?;
    Ok(())
}
