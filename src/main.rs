#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;

use repair_manager;
mod models;

mod database;
use database::db::{DbPool, connect};

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    repair_manager::rocket().await.launch().await;
    Ok(())
}