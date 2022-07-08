#[allow(unused_imports)]

use mongodb::bson::oid::ObjectId;

use mongodb::Collection;
use mongodb::bson::{Document, doc};
use mongodb::options::{FindOptions, CountOptions};
use rocket::time::{OffsetDateTime, Duration};
use serde::{Deserialize, Serialize};
use rocket::http::{Cookie, CookieJar, ContentType};
use rocket::serde::json::Json;
use rocket::{State, Response};

use crate::database;
use crate::models;
use crate::utils;


use database::db::DbPool;
use models::user::User;

use rocket::response::status::{self, Forbidden};

use super::{request_guards::user::UserRequest};


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Customer{
    name: String
}


#[post("/", format="application/json", data="<custo>")]
// pub fn create_customer(custo: Json<Customer>, cookies: &CookieJar<'_>) -> String{
pub fn create_customer(custo: Json<Customer>, user_req: UserRequest, cookies: &CookieJar<'_>) -> String{
    "Ok".to_string()
}