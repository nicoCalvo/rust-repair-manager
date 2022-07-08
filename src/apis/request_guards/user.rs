#[allow(unused_imports)]

use bson::oid::ObjectId;
use rocket::outcome::{try_outcome, Outcome::*};
use rocket::request::{self, Request, FromRequest};
use rocket::response::status::Forbidden;
use rocket::tokio::sync::Mutex;
use rocket::{State, Response};

use crate::database;
use crate::models;

use database::db::{DbPool, connect};
use models::user::User;



pub struct UserRequest{
    id: ObjectId,
    name: String
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserRequest{
    type Error = Forbidden<String>;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        // let con = connect().await;
        println!("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA");
        let connection = request.rocket().state::<DbPool>().unwrap();
        dbg!(&connection.mongo);
        Success(Self{id: ObjectId::new(), name: "pepe".to_string()})


        /*
        obtener el cookie y validar que el user este activo

        */
        // let cookies = request.cookies();
        // let auth_cookie = cookies.get_private("user_id");
        // let user: Option<User> = match auth_cookie{
        //     Some(user) =>{
        //         match connection {
        //             Some(con) => {
        //                 let col = con.mongo.collection("users");


        //                 None

        //             },
        //             None =>{
        //                 None
        //             }

        //         }
                
        //     },
        //     None => Err(Forbidden(Some("Invalid User or password".to_string())))
        // };
    }
}

