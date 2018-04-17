#[macro_use]
extern crate diesel;
#[macro_use] 
extern crate serde_derive;

#[derive(Queryable, Serialize)]
pub struct Item {
    pub id: i32,
    pub title: String,
}