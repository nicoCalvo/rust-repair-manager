use bson::doc;
#[allow(unused_imports)]

use bson::oid::ObjectId;
use mongodb::Collection;
use rocket::outcome::Outcome::{*, self};
use rocket::request::{self, Request, FromRequest};
use rocket::http::Status;

use crate::database;
use crate::models;

use database::db::DbPool;
use models::user::User;


pub struct UserRequest{
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
        // let con = connect().await;

        let pool = request.rocket().state::<DbPool>().unwrap();
        let users_col: Collection<User> = pool.mongo.collection::<User>("users");
        let user_cookie = request.cookies().get_private("user_id");
        if let Some(user_id) = user_cookie {
            match ObjectId::parse_str(user_id.value()){
                Ok(user) =>{
                    let user_res = users_col.find_one(doc!{"_id": user}, None).await.unwrap();
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

