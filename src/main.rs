
//! Example actix-web application.
//!
//! This code is adapted from the front page of the [Actix][] website.
//!
//! [actix]: https://actix.rs/docs/

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use std::env;

// fn greet(req: &HttpRequest) -> impl Responder {
//     let to = req.match_info().get("name").unwrap_or("World");
//     format!("Hello {}!", to)
// }

// fn main() {
//     // Get the port number to listen on.
//     let port = env::var("PORT")
//         .unwrap_or_else(|_| "3000".to_string())
//         .parse()
//         .expect("PORT must be a number");

//     // Start a server, configuring the resources to serve.
//     server::new(|| {
//         App::new()
//             .resource("/", |r| r.f(greet))
//             .resource("/{name}", |r| r.f(greet))
//     })
//     .bind(("0.0.0.0", port))
//     .expect("Can not bind to port 8000")
//     .run();
// }


#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there! How's it going?")
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port: u16 = env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .expect("PORT must be a number");

    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(echo)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(format!("0.0.0.0:{}", port))?
    .run()
    .await
}
