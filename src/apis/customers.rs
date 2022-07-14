#[allow(unused_imports)]

use mongodb::bson::oid::ObjectId;

use mongodb::bson::doc;
use mongodb::options::FindOneOptions;
use mongodb::options::FindOptions;
use rocket::State;
use rocket::http::ext::IntoCollection;
use rocket::response::status;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use rocket::http::Status;
// use  rocket_contrib::json::Json;

use crate::database;
use crate::models;
use crate::models::user::User;


use database::db::DbPool;
use models::customer::Customer;

use rocket::response::status::{Forbidden, };

use super::{request_guards::user::UserRequest};

struct UserId(ObjectId);

#[derive(Serialize, Deserialize)]
pub struct CustomerId{
    _id: String
}


#[derive(Responder)]
enum ApiError {
    #[response(status = 403)]
    Unauthorized(String),
    #[response(status = 422)]
    UnprocesableEntity(String),
    #[response(status = 500)]
    InternalError(String)
}

#[post("/", format="application/json", data="<post_customer>")]
// pub fn create_customer(custo: Json<Customer>, cookies: &CookieJar<'_>) -> String{
pub async fn create_customer(
    post_customer: Json<Customer>,
    user_req: UserRequest,
    db: &State<DbPool>
) -> Result<Json<CustomerId>, ApiError>{
    // check if customer exists based on name, lastname and address and location
    let post_customer = post_customer.into_inner();
    
    let customers_col = db.mongo.collection::<Customer>("customers");
    let _filter = doc!{
        "$and":[
                  {"name": {"$eq": post_customer.name.clone()}},
                  {"last_name": {"$eq": post_customer.last_name.clone()}},
                  {"location": {"$eq": post_customer.location.clone()}},
                  {"street": {"$eq": post_customer.street.clone()}},
                  {"number": {"$eq": post_customer.number.clone()}}
                 
              ]   
        };
    // cannot apply if collection has been tipified as Customer
    // let opt = FindOneOptions::builder().projection(doc!{"_id": 1}).build();
    let customer = customers_col.find_one(_filter, None).await;
    match customer {
        Ok(c) =>{
            let mut _id = String::new();
            if c.is_some(){
                _id = c.unwrap().id.unwrap().to_hex();
                return Err(ApiError::UnprocesableEntity(format!("Customer already exists {}" , _id)))
            }else {
                let result = customers_col.insert_one(post_customer, None).await.unwrap();
                
                _id = result.inserted_id.as_object_id().unwrap().to_hex();
            }
            return Ok(Json(CustomerId{_id}));
        },
        Err(e) =>{
            return Err(ApiError::InternalError("Unable to retrieve customer".to_string()))

        }
    }
    //if not, create and return Id,
    //if so, return Id
    // Ok(Json(custo))
}