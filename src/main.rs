#![feature(proc_macro_hygiene, decl_macro)]
#![allow(unused_imports, dead_code)]
#[macro_use] extern crate rocket;

use repair_manager;
mod models;

mod database;
use database::db::{DbPool, connect};

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    _ = repair_manager::rocket().await.launch().await;
    Ok(())
}