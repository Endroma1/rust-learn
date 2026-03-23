use actix_web::{App, HttpResponse, HttpServer, Responder, get, post, web};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(web::scope("/app").service(scoped_hello))
            .service(hello)
            .service(echo)
            .route("/manual_health", web::get().to(manual_hello))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

#[get("/hello")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("hello")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("hello")
}

#[get("/hello")]
async fn scoped_hello() -> impl Responder {
    HttpResponse::Ok().body("scoped hello")
}
