#[allow(unused_imports)]

use futures::io::Cursor;
use mongodb::bson::oid::ObjectId;

use mongodb::Collection;
use mongodb::bson::{doc};
use rocket::time::{OffsetDateTime, Duration};
use serde::{Deserialize, Serialize};
use rocket::http::{Cookie, CookieJar};
use rocket::serde::json::Json;
use rocket::{State};

use crate::database;
use crate::models;
use crate::utils;

use utils::hash_password;

use database::db::DbPool;
use models::user::User;

use rocket::response::status::{Forbidden};



#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LoginInfo{
    email: String,
    password: String
}


#[post("/", format = "application/json", data = "<login_info>")]
pub async fn login<'r>(
    login_info: Json<LoginInfo>,
    cookies: &CookieJar<'_>,
    mongo_db: &State<DbPool>,
)-> Result<Json<User>, Forbidden<String>>{
    
    let col_users: Collection<User> = mongo_db.mongo.collection("users");
    // set max age to 10 hours
    // check ip origin and add as private attribute http-only

    // Return existing session if valid cookie provided
    if let Some(user_id) =  cookies.get_private("user_id"){
        let user_id = user_id.value();
        
        let user = col_users.find_one(doc!{"_id": ObjectId::parse_str(user_id).unwrap()}, None).await.unwrap();
        match user{
            Some(u) =>{
                let mut cookie = Cookie::new("user_id", u.id.unwrap().to_string());
                let mut now = OffsetDateTime::now_utc();
                now += Duration::hours(10);
                cookie.set_expires(now);
                cookies.add_private(cookie);
                return Ok(Json(u));
            },
            None =>{
                println!("User from cookie does not exists!");
                return Err(Forbidden(Some("Invalid User or password".to_string())))
            }
        }
    }
    // look for the user and check matching password
    let filter = doc!{"email": &login_info.email};
    let user = col_users.find_one(filter, None).await.unwrap();
    match user {
        Some(user) =>{
            // return user as json
            if let Some(_) =  cookies.get_private("user_id"){
                return Ok(Json(user));
            }
            if user.password != hash_password(&login_info.password){
                Err(Forbidden(Some("Invalid User or password".to_string())))
            }else{
                let mut cookie = Cookie::new("user_id", user.id.unwrap().to_string());
                let mut now = OffsetDateTime::now_utc();
                now += Duration::hours(10);
                cookie.set_expires(now);
                cookies.add_private(cookie);
                Ok(Json(user))
            }
        },
        _ => Err(Forbidden(Some("Invalid User or password".to_string())))
    }


}

#[post("/logout")]
pub fn logout(cookies: &CookieJar<'_>) {
    cookies.remove_private(Cookie::named("user_id"));
    
}