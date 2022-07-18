#![allow(dead_code)]
#![allow(unused_imports)]
use rocket::{Rocket, State, Build};
use rocket::fairing::AdHoc;
mod common;


#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use bson::{Document, doc, oid::ObjectId};
    use mongodb::Collection;
    use repair_manager::models::user::User;
    use rocket::{tokio::{self, io::AsyncReadExt}, http::{ContentType, Cookie}, time::{OffsetDateTime, Duration}};
    use ::rocket::{http::Status, async_test};
    use serde_json::Value;
    use ::rocket::local::asynchronous::Client;
    use crate::common::{DbFixture, CustomerBuilder, LoggedClient};

    #[async_test]
    async fn test_create_existing_user() {
        let mut db = DbFixture::new().await;
        let mut client = LoggedClient::init().await;
        client.with_user("create_existing_user@email.com", &mut db, Some("admin".to_string())).await;
        let user = doc!{
            "username":"Matias",
            "email": "create_existing_user@email.com",
            "password": "some_pass",
            "role": "admin"

        };
        let resp = client.post::<Document>(&user, "/users".to_string()).await;
        assert_eq!(resp.status(), Status::UnprocessableEntity);
        db.clean().await;
    }

    #[async_test]
    async fn test_create_user(){
        let mut db = DbFixture::new().await;
        let mut client = LoggedClient::init().await;
        client.with_user("admin@mail.com", &mut db, Some("admin".to_string())).await;
        let users_col = db.db.collection::<User>("users");
        let user = doc!{
            "username":"Matias2",
            "email": "matias2@arrobatech.com.ar",
            "password": "some_pass"

        };
        let resp = client.post::<Document>(&user, "/users".to_string()).await;
        assert_eq!(resp.status(), Status::Ok);
        let asd = resp.into_json::<Document>().await.unwrap();
        let user_objid = ObjectId::parse_str(asd.get_str("_id").unwrap()).unwrap();
        let filter_ = doc!{"_id": user_objid};
        let customer = users_col.find_one(filter_, None).await.unwrap();
        db.clean().await;
        _ = users_col.delete_one(doc!{"_id": user_objid}, None).await;
        match customer{
            Some(c) =>{
                assert!(c.id.unwrap() == user_objid, "User created succesfully")
            },
            None => assert!(false, "failed to create user")
        }
    }
    #[async_test]
    async fn test_create_user_nonadmin(){
        let mut db = DbFixture::new().await;
        let mut client = LoggedClient::init().await;
        client.with_user("user@mail.com", &mut db, None).await;
        _ = db.db.collection::<User>("users");
        let empty = doc!{};
        let resp = client.post::<Document>(&empty, "/users".to_string()).await;
        assert_eq!(resp.status(), Status::Forbidden)
    }

    #[async_test]
    async fn test_get_users() {
        let mut client = LoggedClient::init().await;
        let mut db = DbFixture::new().await;
        client.with_user("test_get_users@email.com", &mut db, Some("admin".to_string())).await;
        let resp = client.get("/users".to_string()).await;
        assert_eq!( resp.status(), Status::Ok);
        let asd = resp.into_json::<Vec<Document>>().await.unwrap();
        assert!(asd.len() >= 1);
        db.clean().await;
    }
}