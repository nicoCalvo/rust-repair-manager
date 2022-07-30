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
    use repair_manager::models::customer::Customer;
    use rocket::{tokio, http::{ContentType, Cookie}, time::{OffsetDateTime, Duration}};
    use ::rocket::{http::Status, async_test};
    use serde_json::Value;
    use ::rocket::local::asynchronous::Client;
    use crate::common::{DbFixture, CustomerBuilder, LoggedClient};

    #[async_test]
    async fn test_create_customer() {
        let mut db = DbFixture::new().await;
        let mut client = LoggedClient::init().await;
        client.with_user("test_create_customer", &mut db, Some("Admin".to_string())).await;
        let customers_col = db.db.collection::<Customer>("customers");
        let customer = Customer{name:"test_create_customer".to_string(), ..Default::default()};

        let resp = client.post::<Customer>(&customer, "/customers".to_string()).await;
        assert_eq!(resp.status(), Status::Ok);
        let asd = resp.into_json::<Document>().await.unwrap();
        let customer_objid = ObjectId::parse_str(asd.get_str("_id").unwrap()).unwrap();
        let filter_ = doc!{"_id": customer_objid};
        let customer = customers_col.find_one(filter_, None).await.unwrap();
        db.clean().await;
        _ = customers_col.delete_one(doc!{"_id": customer_objid}, None).await;
        match customer{
            Some(c) =>{
                assert!(c.id.unwrap() == customer_objid, "Customer created succesfully")
            },
            None => assert!(false, "failed to create customer")
        }
    }   

    #[async_test]
    async fn test_create_existing_customer() {
        let mut db = DbFixture::new().await;
        let cus = Customer{name: "existing_customer".to_string(), ..Default::default()};
        let id = db.create_customer(cus).await;
        let mut client = LoggedClient::init().await;
        client.with_user("test_create_existing_customer", &mut db, Some("Admin".to_string())).await;

        let customer = Customer{name:"existing_customer".to_string(), ..Default::default()};
        let resp = client.post::<Customer>(&customer, "/customers".to_string()).await;
        assert_eq!(resp.status(), Status::UnprocessableEntity);
        let cus_col = db.db.collection::<Customer>("customers");
        _ = cus_col.delete_one(doc!{"_id": ObjectId::parse_str(id).unwrap()}, None);
        db.clean().await
    }

    #[async_test]
    async fn test_forbidden_create_customer() {
        let client = LoggedClient::init().await;
        let customer = Customer{name:"test_forbidden_customer".to_string(), ..Default::default()};
        let resp = client.post::<Customer>(&customer, "/customers".to_string()).await;
        dbg!(&resp);
        assert_eq!(resp.status(), Status::Forbidden);
    }

    #[async_test]
    async fn test_due_cookie() {
        let mut client = LoggedClient::init().await;
        client.due_cookie = true;
        let customer = Customer{name:"test_forbidden_customer".to_string(), ..Default::default()};
        let resp = client.post::<Customer>(&customer, "/customers".to_string()).await;
        assert_eq!(resp.status(), Status::Forbidden)
    }

    #[async_test]
    async fn test_update_customer() {
        let mut client = LoggedClient::init().await;
        let mut db = DbFixture::new().await;
        let cus = Customer{name: "update_customer".to_string(), ..Default::default()};
        let existing_customer = db.create_customer(cus).await;
        client.with_user("test_update_customer", &mut db, Some("Admin".to_string())).await;


        let customers_col = db.db.collection::<Customer>("customers");
        let doc = doc!{
            "id": ObjectId::parse_str(existing_customer).unwrap(),
            "name": "new_name",
            "location": "Valle de tetas"
        };
        let resp = client.put::<Document>(&doc, "/customers".to_string()).await;
        assert_eq!( resp.status(), Status::Ok);
        let asd = resp.into_json::<Document>().await.unwrap();
        let customer_objid = ObjectId::parse_str(asd.get_str("_id").unwrap()).unwrap();
        let filter_ = doc!{"_id": customer_objid};
        let customer = customers_col.find_one(filter_, None).await.unwrap();
        match customer{
            Some(c) =>{
                db.clean().await;
                _ = customers_col.delete_one(doc!{"_id": c.id}, None);
                assert!(c.name == "new_name");
                assert!(c.location == "Valle de tetas")
            },
            None =>{
                db.clean().await;
                assert!(false, "failed to update customer")
            }
        }
    }

  
}