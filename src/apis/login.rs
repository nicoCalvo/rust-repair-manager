use log::info;
use mongodb::bson::oid::ObjectId;

use mongodb::Collection;
use mongodb::bson::{doc};
use rocket::time::{OffsetDateTime, Duration, format_description};
use serde::{Deserialize, Serialize};
use rocket::http::{Cookie, CookieJar};
use rocket::serde::json::{Json, self};
use rocket::serde::json::json;
use rocket::{State};
use rocket::serde::json::Value;

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


// A valid Login injects a private cookie with
// of type: User: {"id": "some id", "role": "Admin"|"Technician"}
// The expiring time is set 10 hours after the login
#[post("/", format = "application/json", data = "<login_info>")]
pub async fn login<'r>(
    login_info: Json<LoginInfo>,
    cookies: &CookieJar<'_>,
    mongo_db: &State<DbPool>,
)-> Result<Json<User>, Forbidden<String>>{
    let col_users: Collection<User> = mongo_db.mongo.collection("users");
    // Return existing session if valid cookie provided
    if let Some(user_cookie) =  cookies.get_private("user"){
        let user_cookie: Value = json::from_str(user_cookie.value()).unwrap();
        let id = user_cookie["id"].as_str().unwrap();
        let user = col_users.find_one(doc!{"_id": ObjectId::parse_str(id).unwrap()}, None).await.unwrap();
        match user{
            
            Some(mut u) =>{
                let mut now = OffsetDateTime::now_utc();
                now += Duration::hours(10);
                let format = format_description::parse(
                    "[year]-[month]-[day] [hour]:[minute]:[second]",
                ).unwrap();
                let ts_string = now.format(&format).unwrap();
                let user_cookie_info = json!({
                    "id": u.id.unwrap().to_string(),
                    "role": u.role,
                    "expires": ts_string
                });
                let mut user_cookie = Cookie::new("user", user_cookie_info.to_string());
                
                user_cookie.set_expires(now);
                user_cookie.set_secure(true);
                user_cookie.http_only();
                cookies.add_private(user_cookie);
                u.password="***".to_string();
                return Ok(Json( u));
            },
            None =>{
                error!("User from cookie does not exists!");
                return Err(Forbidden(Some("Invalid User or password".to_string())))
            }
        }
    }
    // look for the user and check matching password
    let filter = doc!{"email": &login_info.email};
    let user = col_users.find_one(filter, None).await.unwrap();

    match user {
        Some(mut user) =>{
            // return user as json
            if let Some(_) =  cookies.get_private("user"){
                user.password="***".to_string();
                return Ok(Json(user));
            }
            if user.password != hash_password(&login_info.password){
                Err(Forbidden(Some("Invalid User or password".to_string())))
            }else{
                let mut now = OffsetDateTime::now_utc();
                now += Duration::hours(10);
                let format = format_description::parse(
                    "[year]-[month]-[day] [hour]:[minute]:[second]",
                ).unwrap();
                let ts_string = now.format(&format).unwrap();
                let user_cookie_info = json!({
                    "id": user.id.unwrap().to_string(),
                    "role": user.role,
                    "expires": ts_string
                });
                let mut user_cookie = Cookie::new("user", user_cookie_info.to_string());
                user_cookie.set_secure(true);
                user_cookie.set_expires(now);
                cookies.add_private(user_cookie);
                info!("User {} has logged in", user.username);
                user.password="***".to_string();
                return Ok(Json(user));
            }
        },
        _ => {
            info!("invalid login: {:?}", login_info);
            return Err(Forbidden(Some("Invalid User or password".to_string())))
        }
    }


}

#[post("/logout")]
pub fn logout(cookies: &CookieJar<'_>) -> Result<(), Forbidden<String>> {
    cookies.remove_private(Cookie::named("user"));
    return Ok(());
    
}