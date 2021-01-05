#[macro_use]
extern crate sailfish_macros; // enable derive macros
use sailfish::TemplateOnce;
use sqlx::{PgPool, Row};
use sqlx::postgres::PgRow;

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
    use rand::distributions::{Distribution, Uniform};
    use rand::thread_rng;

    const NUM_WORDS: usize = 4;
    // Longest word is 9 letters. Add spaces between each word as well.
    const LONGEST: usize = NUM_WORDS * 9 + (NUM_WORDS - 1);
    static FILE: &str = include_str!("../wordlist.txt");

    pub fn get_words() -> String {
        let wordvec: Vec<&str> = FILE.split_whitespace().collect();

        let mut ret: String = String::with_capacity(LONGEST);

        let mut rng = thread_rng();
        let dist = Uniform::from(0..wordvec.len());

        for w in 0..NUM_WORDS {
            let word: &str = wordvec[dist.sample(&mut rng)];
            ret.push_str(word);

            if w < NUM_WORDS - 1 {
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

struct WordMap {
    words: String,
    link: String,
}

#[post("/new")]
async fn newlink(
    mut req_body: web::Form<model::MainForm>,
    db_pool: web::Data<PgPool>,
) -> impl Responder {

    let map = WordMap {
        words: helper::get_words(),
        // Consume the form to get the query. Avoids cloning the query unecessarily.
        // There is probably a better way to do this that I haven't found yet.
        link: std::mem::replace(&mut req_body.query, String::new()),
    };

    let mut tx = db_pool.begin().await.unwrap();
    let map: WordMap = sqlx::query("INSERT INTO wordmap (words, link) VALUES ($1, $2) RETURNING words, link")
        .bind(&map.words)
        .bind(&map.link)
        .map(|row: PgRow| { 
            WordMap {
                words: row.get(0), 
                link: row.get(1),
            }
        })
        .fetch_one(&mut tx)
        .await.unwrap();
    tx.commit().await.unwrap();

    // TODO write your own unwrap function (or find one in actix)
    // that returns a 500 error code instead of crashing.
    let response = model::PostedTemplate {
        content: format!("link:{}, words:{}", map.link, map.words),
    }
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .expect("PORT must be a number");

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set");
    // let test: &str = &database_url;
    let db_pool = PgPool::connect(&database_url)
        .await
        .expect("Error opening postgres database.");

    println!("Starting server on port {}...", port);

    HttpServer::new(move || {
        App::new()
            .data(db_pool.clone())
            .service(index)
            .service(newlink)
            .service(redir)
    })
    .bind(format!("0.0.0.0:{}", port))?
    .run()
    .await
}
