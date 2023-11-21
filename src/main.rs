use std::{env, ptr::NonNull};

use uuid::Uuid;

use sqlx::{Connection, Row, PgPool, Pool, query_as};
use serde::{Deserialize, Serialize};
use tide::{Body, Request, Response, Server};

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
struct Book {
    id: sqlx::types::Uuid,
    name: Option<String>,
    author: Option<String>,
    year: Option<i32>
}

#[derive(Clone,Debug)]
struct State {
    db_pool: PgPool
}

#[async_std::main]
async fn main() -> Result<(), std::io::Error>{
    tide::log::start();
    
    let db_pool = make_db_pool().await;
    let app = server(db_pool).await;

    app.listen("127.0.0.1:8080").await.unwrap();

    // let dbuser = env::var("PGUSER").unwrap_or("postgres".to_string());
    // let dbpwd = env::var("PGPWD").unwrap_or("postgres".to_string());
    // let dbhost = env::var("PGHOST").unwrap_or("localhost:5432".to_string());
    // let dbschema = env::var("PGSCHEMA").unwrap_or("a".to_string());

    // let url = format!("postgres://{}:{}@{}/{}", dbuser, dbpwd, dbhost, dbschema);
    // let mut conn = sqlx::postgres::PgConnection::connect(&url).await?;

    // let res = sqlx::query("SELECT 1+1 as sum")
    //     .fetch_one(&mut conn)
    //     .await?;

    // let sum: i32 = res.get("sum");
    // println!("1 + 1 = {}", sum);

    Ok(())
}

pub async fn make_db_pool() -> PgPool {
    let db_url = env::var("DATABASE_URL").unwrap_or(String::from("postgres://postgres:postgres@localhost:5432/rust_crud"));
    Pool::connect(&db_url).await.unwrap()
}

async fn server(book_store: PgPool) -> Server<State> {
    let state = State {
        db_pool: book_store
    };

    let mut app = tide::with_state(state);
    app.at("/").get(|_| async {Ok("Hello, world!")});

    app.at("/books")
        .post(create_book)
        .get(list_books);

    app.at("/books/:id")
        .get(get_book)
        .put(update_book)
        .delete(delete_book);

    app

}

async fn create_book(mut req: Request<State>) -> tide::Result {
    let book: Book = req.body_json().await?;
    let db_pool = req.state().db_pool.clone();
    let row = query_as::<_, Book>(
        r#"
        INSERT INTO book (id, name, author, year)
        VALUES ($1, $2, $3, $4)
        RETURNING id, name, author, year
        "#)
        .bind(book.id)
        .bind(book.name)
        .bind(book.author)
        .bind(book.year)
        .fetch_one(&db_pool).await?;

    // ALTERNATIVE using the macro
    // let row = query_as!(Book,
    //     r#"
    //     INSERT INTO book (id, name, author, year)
    //     VALUES ($1, $2, $3, $4)
    //     returning id, name, author, year
    //     "#,
    //     book.id,
    //     book.name,
    //     book.author,
    //     book.year)
    //     .fetch_one(&db_pool).await?;

    let mut res = Response::new(201);
    res.set_body(Body::from_json(&row)?);
    Ok(res)
}

async fn list_books(req: tide::Request<State>) -> tide::Result {
    let db_pool = req.state().db_pool.clone();
    let rows = query_as::<_, Book>(
        r#"
        SELECT * FROM book
        "#)
        .fetch_all(&db_pool).await?;

    let mut res = Response::new(200);
    res.set_body(Body::from_json(&rows)?);
    Ok(res)
}

async fn get_book(req: tide::Request<State>) -> tide::Result {
    let db_pool = req.state().db_pool.clone();
    let id: Uuid = Uuid::parse_str(req.param("id")?).unwrap();
    let row = query_as::<_, Book>(
        r#"
        SELECT * FROM book
        WHERE id = $1
        "#)
        .bind(id)
        .fetch_optional(&db_pool).await?;

    let mut res = Response::new(200);
    res.set_body(Body::from_json(&row)?);
    Ok(res)
}

async fn update_book(mut req: tide::Request<State>) -> tide::Result {
    let book: Book = req.body_json().await?;
    let db_pool = req.state().db_pool.clone();
    let id: Uuid = Uuid::parse_str(req.param("id")?).unwrap();
    let row = query_as::<_, Book>(
        r#"
        UPDATE book
        SET name = $2, author = $3, year = $4
        WHERE id = $1
        RETURNING id, name, author, year
        "#)
        .bind(id)
        .bind(book.name)
        .bind(book.author)
        .bind(book.year)
        .fetch_optional(&db_pool).await?;

    let mut res = Response::new(200);
    res.set_body(Body::from_json(&row)?);
    Ok(res)
}

async fn delete_book(req: tide::Request<State>) -> tide::Result {
    let db_pool = req.state().db_pool.clone();
    let id: Uuid = Uuid::parse_str(req.param("id")?).unwrap();
    let row = query_as::<_, Book>(
        r#"
        DELETE FROM book
        WHERE id = $1
        "#)
        .bind(id)
        .fetch_optional(&db_pool).await?;

    let mut res = Response::new(200);
    res.set_body(Body::from_json(&row)?);
    Ok(res)
}

// #[async_std::test]
// async fn list_books() -> tide::Result<()> {
//     use tide::http::{Method, Request, Response, Url};

//     let book = Book {
//         name: String::from("The Rust Programming Language"),
//         author: String::from("Steve Klabnik, Carol Nichols"),
//         year: 2018
//     };

//      let mut book_store =
// }