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

mod helper {
    use rand::{thread_rng};
    use rand::distributions::{Distribution, Uniform};

    const NUM_WORDS: usize = 4;
    static FILE: &str = include_str!("../wordlist.txt");

    pub fn get_words() -> String {
        let wordvec: Vec<&str> = FILE.split_whitespace().collect();

        // Longest word is 9 letters. Add spaces between each word as well.
        let longest_possible = NUM_WORDS * 9 + (NUM_WORDS - 1);
        let mut ret: String = String::with_capacity(longest_possible);

        let mut rng = thread_rng();
        let dist = Uniform::from(0..wordvec.len());

        for w in 0..NUM_WORDS {
            let word: &str = wordvec[dist.sample(&mut rng)];
            ret.push_str(word);

            if w < NUM_WORDS-1 {
                ret.push(' ');
            }
        }

        ret
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
    let response = model::PostedTemplate { content: format!("query:{}, response:{}", query, helper::get_words()) }
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

    println!("Starting server on port {}...", port);

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
