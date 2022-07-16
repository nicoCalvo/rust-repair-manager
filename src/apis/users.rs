#![allow(unused_imports, dead_code)]
// create-restore-delete(deactivate only)
// admin only

use bson::to_document;
use mongodb::bson::doc;
use rocket::State;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

use crate::database;
use crate::models;

use database::db::DbPool;
use models::user::User;
use super::{request_guards::user::AdminRequest};


#[derive(Serialize, Deserialize)]
pub struct UserId{
    _id: String
}


#[derive(Responder)]
pub enum ApiError {
    #[response(status = 422)]
    UnprocesableEntity(String),
    #[response(status = 500)]
    InternalError(String)
}

#[post("/", format="application/json", data="<post_customer>")]
pub async fn create_customer(
    post_customer: Json<Customer>,
    _admin_req: AdminRequest,
    db: &State<DbPool>
) -> Result<Json<UserId>, ApiError>{
    

}