#![feature(plugin)]
#![plugin(rocket_codegen)]


extern crate todomvc;

extern crate rocket;
extern crate rocket_contrib;
extern crate core;
#[macro_use] 
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
#[macro_use] 
extern crate diesel;
extern crate dotenv;
#[macro_use] extern crate dotenv_codegen;
extern crate todomvc_models;

use rocket_contrib::{Template, Json};
use rocket::response::NamedFile;
use rocket::request::{self, FromRequest};
use rocket::{Request, State, Outcome};
use rocket::http::Status;
use std::ops::Deref;
use std::path::Path;
use dotenv::dotenv;
use std::env;
// use std::sync::atomic::AtomicUsize;
// use core::sync::atomic::Ordering;
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};

use self::todomvc::*;
use self::diesel::prelude::*;
use todomvc_models::*;


// An alias to the type for a pool of Diesel SQLite connections.
type PgPool = Pool<ConnectionManager<PgConnection>>;

static DATABASE_URL: &'static str = dotenv!("DATABASE_URL");

fn init_pool() -> PgPool {
    let manager = ConnectionManager::<PgConnection>::new(DATABASE_URL);
    Pool::new(manager).expect("db pool")
}

pub struct DbConn(pub PooledConnection<ConnectionManager<PgConnection>>);

/// Attempts to retrieve a single connection from the managed database pool. If
/// no pool is currently managed, fails with an `InternalServerError` status. If
/// no connections are available, fails with a `ServiceUnavailable` status.
impl<'a, 'r> FromRequest<'a, 'r> for DbConn {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let pool = request.guard::<State<PgPool>>()?;
        match pool.get() {
            Ok(conn) => Outcome::Success(DbConn(conn)),
            Err(_) => Outcome::Failure((Status::ServiceUnavailable, ()))
        }
    }
}

// For the convenience of using an &DbConn as an &SqliteConnection.
impl Deref for DbConn {
    type Target = PgConnection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}


#[get("/")]
fn index() -> Template {
    let context = "";
    Template::render("index", &context)
}

#[get("/counter.json")]
fn get_counter(conn: DbConn) -> QueryResult<Json<Vec<Item>>> {
    use todomvc::schema::items::dsl::*;

    items
    .limit(5)
    .load::<Item>(&*conn)
    .map(|item| Json(item))
}

#[get("/app.js")]
fn app() -> Option<NamedFile> {
    NamedFile::open(Path::new("client/target/asmjs-unknown-emscripten/debug/client.js")).ok()
}

fn main() {
    rocket::ignite()
    .mount("/", routes![index, app, get_counter])
    .attach(Template::fairing())
    .manage(init_pool())
    .launch();
}