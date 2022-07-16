#![allow(unused_imports, dead_code)]
use bson::doc;


use bson::oid::ObjectId;
use mongodb::Collection;
use rocket::outcome::Outcome::{*, self};
use rocket::request::{self, Request, FromRequest};
use rocket::http::Status;
use rocket::serde::json;
use rocket::serde::json::Value;
use rocket::serde::json::json;

use crate::database;
use crate::models;

use database::db::DbPool;
use models::user::User;


pub struct UserRequest{
    id: ObjectId,
    name: String,
}

pub struct AdminRequest{
    id: ObjectId,
    name: String
}

#[derive(Debug)]
pub enum AuthCookieError {
    Missing,
    Invalid,
}


#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserRequest{
    type Error = AuthCookieError;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {

        let pool = request.rocket().state::<DbPool>().unwrap();
        let users_col: Collection<User> = pool.mongo.collection::<User>("users");
        let user_cookie = request.cookies().get_private("user");
        if let Some(user) = user_cookie {
            let user: Value = json::from_str(user.value()).unwrap_or(json!({"id": ""}));
            let id = user["id"].as_str().unwrap();
            match ObjectId::parse_str(id){
                Ok(user) =>{
                    let user_res = users_col.find_one(doc!{"_id": user, "active": true}, None).await.unwrap();
                    if let Some(user_obj) = user_res{
                        Success(Self{id: user_obj.id.unwrap(), name: user_obj.username})
                    }
                    else{
                        Outcome::Failure((Status::Forbidden, AuthCookieError::Invalid))
                    }
                },
                Err(_) =>{
                    Outcome::Failure((Status::Forbidden, AuthCookieError::Missing))
                }
            }
        }else{
            Outcome::Failure((Status::Forbidden, AuthCookieError::Missing))
        }
    }
}



#[rocket::async_trait]
impl<'r> FromRequest<'r> for AdminRequest{
    type Error = AuthCookieError;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {

        let pool = request.rocket().state::<DbPool>().unwrap();
        let users_col: Collection<User> = pool.mongo.collection::<User>("users");
        let user_cookie = request.cookies().get_private("user");
        if let Some(user) = user_cookie {
            let user: Value = json::from_str(user.value()).unwrap_or(json!({"id": ""}));
            let id = user["id"].as_str().unwrap();
            match ObjectId::parse_str(id){
                Ok(user) =>{
                    let admin_filter = doc!{"_id": user, "role": "admin", "active": true};
                    let user_res = users_col.find_one(admin_filter, None).await.unwrap();
                    if let Some(user_obj) = user_res{
                        Success(Self{id: user_obj.id.unwrap(), name: user_obj.username})
                    }
                    else{
                        Outcome::Failure((Status::Forbidden, AuthCookieError::Invalid))
                    }
                },
                Err(_) =>{
                    Outcome::Failure((Status::Forbidden, AuthCookieError::Missing))
                }
            }
        }else{
            Outcome::Failure((Status::Forbidden, AuthCookieError::Missing))
        }
    }
}

