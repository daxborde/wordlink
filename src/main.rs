#[macro_use]
extern crate sailfish_macros; // enable derive macros

mod model {
    use serde::{Deserialize, Serialize};

    // Form
    #[derive(Serialize, Deserialize)]
    pub struct MainForm {
        pub query: String,
    }

    // Query Successful Template
    #[derive(TemplateOnce)]
    #[template(path = "query.stpl")]
    pub struct QueryTemplate {
        pub link: String,
        pub words: String,
    }

    // Insert new link Template
    #[derive(TemplateOnce)]
    #[template(path = "insert.stpl")]
    pub struct InsertTemplate {
        pub words: String,
        pub link: String,
    }
}

mod helper {
    use rand::distributions::{Distribution, Uniform};
    use rand::thread_rng;
    use std::collections::HashSet;

    const NUM_WORDS: usize = 4;
    // Longest word is 9 letters. Add spaces between each word as well.
    const LONGEST: usize = NUM_WORDS * 9 + (NUM_WORDS - 1);
    static FILE: &str = include_str!("../wordlist.txt");

    pub fn is_words(input: &str) -> bool {
        let wordvec: HashSet<&str> = FILE.split_whitespace().collect();
        // let words = input.split(' ');
        let result = input.split(' ').count() == NUM_WORDS
            && !input
                .split(' ')
                .any(|word| word.chars().any(|x| !x.is_alphabetic()))
            && !input.split(' ').any(|s| !wordvec.contains(s));
        result
    }

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

mod db {
    use sqlx::postgres::PgRow;
    use sqlx::{PgPool, Row};

    use actix_web::web::Data;

    #[derive(sqlx::FromRow)]
    pub struct WordMap {
        pub words: String,
        pub link: String,
    }

    pub async fn insert_wordmap(map: &WordMap, db_pool: Data<PgPool>) -> WordMap {
        let mut tx = db_pool.begin().await.unwrap();
        let map: WordMap =
            sqlx::query("INSERT INTO wordmap (words, link) VALUES ($1, $2) RETURNING words, link")
                .bind(&map.words)
                .bind(&map.link)
                .map(|row: PgRow| WordMap {
                    words: row.get(0),
                    link: row.get(1),
                })
                .fetch_one(&mut tx)
                .await
                .unwrap();
        tx.commit().await.unwrap();
        map
    }

    pub async fn query_words(words: &str, db_pool: Data<PgPool>) -> WordMap {
        let mut tx = db_pool.begin().await.unwrap();

        let map: WordMap = sqlx::query_as("SELECT words, link FROM wordmap WHERE words=$1")
            .bind(words)
            .fetch_one(&mut tx)
            .await
            .unwrap();

        tx.commit().await.unwrap();
        map
    }
}

use actix_files as fs;
use actix_web::{get, http, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use sailfish::TemplateOnce;
use sqlx::{PgPool, Pool, Postgres};
use std::env;

#[post("/new")]
async fn newlink(
    req_body: web::Form<model::MainForm>,
    db_pool: web::Data<Pool<Postgres>>,
) -> impl Responder {
    let query = req_body.query.trim();

    let rendered = if helper::is_words(query) {
        let final_map = db::query_words(query, db_pool).await;

        model::QueryTemplate {
            link: final_map.link,
            words: final_map.words,
        }
        .render_once()
        .unwrap()
    } else {
        let map = db::WordMap {
            words: helper::get_words(),
            link: query.to_string(),
        };

        let final_map = db::insert_wordmap(&map, db_pool).await;

        // TODO write your own unwrap function (or find one in actix)
        // that returns a 500 error code instead of crashing when rendering the
        // template fails.
        model::InsertTemplate {
            link: final_map.link,
            words: final_map.words,
        }
        .render_once()
        .unwrap()
    };

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(rendered)
}

#[get("/redir")]
async fn redir() -> impl Responder {
    HttpResponse::Found()
        .header(http::header::LOCATION, "http://example.com")
        .body("<a href=\"http://example.com/\">http://example.com/</a>")
}

#[get("/")]
async fn index(req: HttpRequest) -> impl Responder {
    fs::NamedFile::open("static/index.html")?.into_response(&req)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .expect("PORT must be a number");

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set");
    let db_pool = connect_and_test_db(&database_url).await;

    println!("Starting server on port {}...", port);

    HttpServer::new(move || {
        App::new()
            .data(db_pool.clone())
            .service(index)
            .service(fs::Files::new("/static", "static/").index_file("index.html"))
            .service(newlink)
            .service(redir)
    })
    .bind(format!("0.0.0.0:{}", port))?
    .run()
    .await
}

async fn connect_and_test_db(database_url: &str) -> Pool<Postgres> {
    let db_pool_mut = match PgPool::connect(database_url).await {
        Ok(x) => x,
        Err(e) => panic!(
            "Opening the database failed.\n\
            Please check that the DB is running and that DATABASE_URL is correct.\n\
            My current value for DATABASE_URL is {}\n\
            Additional info: {}",
            database_url, e
        ),
    };
    let db_pool = &db_pool_mut;

    let _result: (i64,) = match sqlx::query_as("SELECT $1")
        .bind(567_i64)
        .fetch_one(db_pool)
        .await
    {
        Ok(x) => x,
        Err(e) => panic!(
            "DB was opened, but failed on a basic query.\n\
            Please check that the DB is running and that DATABASE_URL is correct.\n\
            My current value for DATABASE_URL is {}\n\
            Additional info: {}",
            database_url, e
        ),
    };

    match sqlx::query("SELECT * FROM wordmap")
        .fetch_optional(db_pool)
        .await
    {
        Ok(_) => {}
        Err(_) => create_wordmap_table(db_pool).await,
    };

    db_pool_mut
}

async fn create_wordmap_table(db_pool: &Pool<Postgres>) {
    match sqlx::query(
        "
CREATE TABLE IF NOT EXISTS wordmap (
    id      SERIAL PRIMARY KEY,
    words   VARCHAR(40) NOT NULL,
    link    TEXT NOT NULL
);
    ",
    )
    .fetch_optional(db_pool)
    .await
    {
        Ok(_) => {}
        Err(e) => panic!(
            "Database was opened, but the \"wordmap\" table was not found, and creating it failed.\n\
            Check that the database being used is Postgres and is not in an error state.\n\
            Additional info: {}",
            e
        ),
    }
}
