#![feature(proc_macro_hygiene, decl_macro)]
#[allow(unused_imports, dead_code)]
#[macro_use]
extern crate rocket;


pub mod database;
use database::db::{DbPool, connect};

mod apis;
pub mod models;


pub mod utils;
use utils::logger::setup_logger;

mod fairings;

#[options("/<_..>")]
fn all_options() {
    /* Intentionally left empty */
}


#[launch]
pub async fn rocket() -> _ {
    _ = setup_logger();
    let db = connect().await;
    let timer = fairings::request_timer::RequestTimer{};
    let cors = fairings::cors::CORS{};
    let _rocket = rocket::build()
    .manage(DbPool { mongo: db })
    .attach(timer)
    .attach(cors)
    .mount("/", routes![all_options])
    .mount("/login", routes![apis::login::login])
    .mount("/logout", routes![apis::login::logout])
    .mount("/repairs", routes![
        apis::repairs::create_repair,
        apis::repairs::update_repair,
        apis::repairs::repair_string,
        apis::repairs::repair_int,
        apis::repairs::catalog,
        apis::repairs::product_types,
        apis::repairs::customer_repairs])
    .mount("/customers", routes![apis::customers::create_customer, apis::customers::update_customer, apis::customers::get_customers])
    .mount("/users", routes![apis::users::create_user, apis::users::get_users, apis::users::activate_user]);
    _rocket
}