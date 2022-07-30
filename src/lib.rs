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



#[launch]
pub async fn rocket() -> _ {
    _ = setup_logger();
    // info!("YOLO!");
    // aca hacer un get config y ver si es debug o no para setear la url
    let db = connect().await;
    let auth_fair = fairings::request_timer::RequestTimer{};
    let _rocket = rocket::build()
    .manage(DbPool { mongo: db })
    .attach(auth_fair)
    .mount("/login", routes![apis::login::login])
    .mount("/logout", routes![apis::login::logout])
    .mount("/repairs", routes![apis::repairs::create_repair, apis::repairs::update_repair])
    .mount("/customers", routes![apis::customers::create_customer, apis::customers::update_customer])
    .mount("/users", routes![apis::users::create_user, apis::users::get_users]);
    _rocket
}