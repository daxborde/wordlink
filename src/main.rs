#[macro_use]
extern crate sailfish_macros; // enable derive macros
use sailfish::TemplateOnce;

mod model {
    use serde::{Deserialize, Serialize};

    // Form
    #[derive(Serialize, Deserialize)]
    pub struct MainForm {
        pub query: String,
    }

    // Response Template
    #[derive(TemplateOnce)]
    #[template(path = "posted.stpl")]
    pub struct PostedTemplate {
        pub content: String,
    }
}

use actix_web::{get, http, post, web, App, HttpResponse, HttpServer, Responder};
use std::env;

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../templates/index.html"))
}

#[post("/echo")]
async fn echo(mut req_body: web::Form<model::MainForm>) -> impl Responder {
    // Consume the form to get the query. Avoids cloning the query unecessarily.
    // There is probably a better way to do this that I haven't found yet.
    let query = std::mem::replace(&mut req_body.query, String::new());
    // let query = req_body.query.clone();

    // TODO write your own unwrap function (or find one in actix)
    // that returns a 500 error code instead of crashing.
    let response = model::PostedTemplate { content: query }
        .render_once()
        .unwrap();

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(response)
}

#[get("/redir")]
async fn redir() -> impl Responder {
    HttpResponse::Found()
        .header(http::header::LOCATION, "http://example.com")
        .body("<a href=\"http://example.com/\">http://example.com/</a>")
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
            .service(index)
            .service(echo)
            .service(redir)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(format!("0.0.0.0:{}", port))?
    .run()
    .await
}
