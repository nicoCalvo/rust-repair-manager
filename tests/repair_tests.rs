#![allow(dead_code, unused_variables)]
#![allow(unused_imports)]
use rocket::{Rocket, State, Build};
use rocket::fairing::AdHoc;


mod common;

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use std::str::FromStr;

    use bson::Bson;
    use bson::{Document, doc, oid::ObjectId, DateTime};
    use chrono::Utc;
    use mongodb::Collection;
    use repair_manager::models::repair::{self, Repair, RepairState, Log};
    use repair_manager::models::customer::{self, Customer};
    use rocket::{tokio::{self, io::AsyncReadExt}, http::ContentType, time::OffsetDateTime};
    use ::rocket::{http::Status, async_test};
    use serde::{Serialize, Deserialize};
    use super::Rocket;
    use ::rocket::local::asynchronous::Client;
    use crate::common::{DbFixture, CustomerBuilder, LoggedClient, create_dummy_repair};


    #[async_test]
    async fn test_create_repair_new_customer() {
        let mut db = DbFixture::new().await;
        let mut client = LoggedClient::init().await;
        client.with_user("test_create_repair_new_cus", &mut db, Some("Admin".to_string())).await;
        let repair_request = doc!{
            "customer":{
                "name": "test_create_repair_customer",
                "last_name": "some_last_name",
                "location": "white bay",
                "address": "avenida siempre viva 2",
                "phone": "12345",
                "email": "si"
            },
            "product":{
                "product_type": "cellphone",
                "brand": "Samsung",
                "model": "asd-123",
                "serial_number": "123-dfsdfds",
                "created_at": Utc::now().to_string()
            },
            "description": "No le anda",
            "additional": "si",
            "suggested_price": 23,
            "warranty": 6,
            "estimated_fixed_date": Utc::now().date().format("%Y-%m-%d").to_string(),
            "received_date": Utc::now().to_string()
        };
        
        let res = client.post(&repair_request, "/repairs/repair".to_string()).await;
        assert_eq!(res.status(), Status::Ok);
        let res = res.into_json::<Document>().await.unwrap();
        let repair_id = res.get_object_id("repair_id").unwrap();
        let customer_id = res.get_object_id("customer_id").unwrap();
        let repairs_col = db.db.collection::<Repair>("repairs");
        let created_repair = repairs_col.find_one(doc!{"_id": repair_id}, None).await.unwrap();
        assert!(created_repair.is_some());
        let created_repair = created_repair.unwrap();
        assert_eq!(created_repair.customer, customer_id);
        assert_eq!(created_repair.product.product_type, "cellphone".to_string());
        assert_eq!(created_repair.product.brand, "Samsung".to_string());
        assert_eq!(created_repair.product.model, "asd-123".to_string());
        assert_eq!(created_repair.status, RepairState::Received);

        let cus_col = db.db.collection::<Document>("customers");
         _ = cus_col.delete_one(doc!{"_id": customer_id}, None).await;
        _ = repairs_col.delete_one(doc!{"_id": repair_id}, None).await;
        


    }

    #[async_test]
    async fn test_create_repair_existing_customer_new_product() {
        let mut db = DbFixture::new().await;
        let mut client = LoggedClient::init().await;
        client.with_user("test_create_repair_new_customer", &mut db, Some("Admin".to_string())).await;
        let repair_request = doc!{
            "customer":{
                "name": "test_create_repair_existing_customer_new_product",
                "last_name": "some_last_name",
                "location": "white bay",
                "address": "avenida siempre viva 2",
                "phone": "12345",
                "email": "si"
            },
            "product":{
                "product_type": "cellphone",
                "brand": "Samsung",
                "model": "asd-123",
                "serial_number": "123-dfsdfds",
                "created_at": Utc::now().to_string()
            },
            "description": "No le anda",
            "additional": "si",
            "suggested_price": 23,
            "warranty": 6,
            "estimated_fixed_date": Utc::now().date().format("%Y-%m-%d").to_string(),
            "received_date": Utc::now().to_string()
        };
        
        let res = client.post(&repair_request, "/repairs/repair".to_string()).await;
        assert_eq!(res.status(), Status::Ok);
        let res = res.into_json::<Document>().await.unwrap();
        let repair_id = res.get_object_id("repair_id").unwrap();
        let customer_id = res.get_object_id("customer_id").unwrap();
        let repair_request2 = doc!{
            "customer":{
                "name": "test_create_repair_existing_customer_new_product",
                "last_name": "some_last_name",
                "location": "white bay",
                "address": "avenida siempre viva 2",
                "phone": "12345",
                "email": "si"
            },
            "product":{
                "product_type": "cellphone",
                "brand": "Samsung",
                "model": "asd-123",
                "serial_number": "123-dfsdfds-XXX",
                "created_at": Utc::now().to_string()
            },
            "description": "No le anda",
            "additional": "si",
            "suggested_price": 23,
            "warranty": 6,
            "estimated_fixed_date": Utc::now().date().format("%Y-%m-%d").to_string(),
            "received_date": Utc::now().to_string()
        };
        let res = client.post(&repair_request2, "/repairs/repair".to_string()).await;
        let res = res.into_json::<Document>().await.unwrap();
        let repair_id2 = res.get_object_id("repair_id").unwrap();

        let repairs_col = db.db.collection::<Repair>("repairs");
        let cus_col = db.db.collection::<Customer>("customers");
        let cus: Customer = cus_col.find_one(doc!{"_id": customer_id}, None).await.unwrap().unwrap();
        assert_eq!(cus.repaired_products.len(), 2);
        _ = cus_col.delete_one(doc!{"_id": customer_id}, None).await;
        _ = repairs_col.delete_one(doc!{"_id": repair_id}, None).await;
        _ = repairs_col.delete_one(doc!{"_id": repair_id2}, None).await;
        

    }
    
    #[async_test]
    async fn test_in_progress_repair() {
        let mut db = DbFixture::new().await;
        let mut client = LoggedClient::init().await;
        client.with_user("test_create_repair_new_cus", &mut db, Some("Admin".to_string())).await;
        let repair_request = doc!{
            "customer":{
                "name": "test_in_progress_repair",
                "last_name": "some_last_name",
                "location": "white bay",
                "address": "avenida siempre viva 2",
                "phone": "12345",
                "email": "si"
            },
            "product":{
                "product_type": "cellphone",
                "brand": "Samsung",
                "model": "asd-123",
                "serial_number": "123-dfsdfds",
                "created_at": Utc::now().to_string()
            },
            "description": "No le anda",
            "additional": "si",
            "suggested_price": 23,
            "warranty": 6,
            "estimated_fixed_date": Utc::now().date().format("%Y-%m-%d").to_string(),
            "received_date": Utc::now().to_string()
        };
        
        let res = client.post(&repair_request, "/repairs/repair".to_string()).await;
        assert_eq!(res.status(), Status::Ok);
        let res = res.into_json::<Document>().await.unwrap();
        let repair_id = res.get_object_id("repair_id").unwrap();
        let customer_id = res.get_object_id("customer_id").unwrap();
        // aca validar que la repair exista y luego borrarla
        let repair_request = doc!{
            "customer":{
                "name": "test_in_progress_repair",
                "last_name": "some_last_name",
                "location": "white bay",
                "address": "avenida siempre viva 2",
                "phone": "12345",
                "email": "si"
            },
            "product":{
                "product_type": "cellphone",
                "brand": "Samsung",
                "model": "asd-123",
                "serial_number": "123-dfsdfds-XXX",
                "created_at": Utc::now().to_string()
            },
            "description": "No le anda",
            "additional": "si",
            "suggested_price": 23,
            "warranty": 6,
            "estimated_fixed_date": Utc::now().date().format("%Y-%m-%d").to_string(),
            "received_date": Utc::now().to_string()
        };

        let res = client.post(&repair_request, "/repairs/repair".to_string()).await;
        let res = res.into_json::<Document>().await.unwrap();
        let repair_id2 = res.get_object_id("repair_id").unwrap();

        let repairs_col = db.db.collection::<Repair>("repairs");
        let cus_col = db.db.collection::<Customer>("customers");
        let cus: Customer = cus_col.find_one(doc!{"_id": customer_id}, None).await.unwrap().unwrap();
        assert!(cus.repaired_products.len() >= 2);
        _ = cus_col.delete_one(doc!{"_id": customer_id}, None).await;
        _ = repairs_col.delete_one(doc!{"_id": repair_id}, None).await;
        _ = repairs_col.delete_one(doc!{"_id": repair_id2}, None).await;
        

    }

    #[async_test]
    async fn test_update_repair() {
        let mut db = DbFixture::new().await;
        let mut client = LoggedClient::init().await;
        let user_id: String = client.with_user("test_update_repair", &mut db, Some("Admin".to_string())).await;
        let user_id = ObjectId::from_str(&user_id).unwrap();
        
        // test unexistent repair
        let repair_request = doc!{
            "repair_id": ObjectId::new(),
            "status": "En progreso",
            "log_entry": "yolo"
        };
        let res = client.put(&repair_request, "/repairs/repair".to_string()).await;
        assert_eq!(res.status(), Status::NotFound);

        // test Received repair into progress
        let log_entry = Log{ entry: "Recibida".to_string(), status: RepairState::Received, created_at: Utc::now(), by: "Someone".to_string()};
        let res = create_dummy_repair(&user_id, &db.db, "cellphone".to_string(),"Recibida".to_string(), &user_id, log_entry).await;
        let rep_id = res.0;  
        let cus_id = res.1;
        let repair_request = doc!{
            "repair_id": rep_id,
            "status": "En progreso",
            "log_entry": "working on it"
        };
        let resp = client.put(&repair_request, "/repairs/repair".to_string()).await;
        assert_eq!(resp.status(), Status::Ok);

        let repair_request = doc!{
            "repair_id": rep_id,
            "status": "Para entregar",
            "log_entry": "se cambio pitutito",
            "suggested_price": 4343,
            "warranty": 12
        };
        let res = client.put(&repair_request, "/repairs/repair".to_string()).await;

        let repair_request = doc!{
            "repair_id": rep_id,
            "status": "Entregada",
            "bill":{
                "amount": 3456,
                "pay_method": "credit"
            }
        };
        let res = client.put(&repair_request, "/repairs/repair".to_string()).await;
        assert_eq!(resp.status(), Status::Ok);

        let repair_request = doc!{
            "repair_id": rep_id,
            "status": "Anulada",
        };
        let res = client.put(&repair_request, "/repairs/repair".to_string()).await;
        assert_eq!(resp.status(), Status::Ok);


        let reps_col = db.db.collection::<Repair>("repairs");
        let cus_col = db.db.collection::<Document>("customers");
        _ = reps_col.delete_one(doc!{"_id": rep_id}, None).await;
        _ = cus_col.delete_one(doc!{"_id": cus_id}, None).await;
    }


    #[async_test]
    async fn test_get_repair() {
        let mut db = DbFixture::new().await;
        let mut client = LoggedClient::init().await;
        let user_id: String = client.with_user("test_update_repair", &mut db, Some("Admin".to_string())).await;
        let user_id = ObjectId::from_str(&user_id).unwrap();
        let log_entry = Log{ entry: "Recibida".to_string(), status: RepairState::Received, created_at: Utc::now(), by: "Someone".to_string()};
        let res = create_dummy_repair(&user_id, &db.db, "cellphone".to_string(),"Recibida".to_string(), &user_id, log_entry).await;
        let rep_id = res.0;  
        let cus_id = res.1;
    
        let res = client.get(format!("/repairs/repair/{}", rep_id).to_string()).await;
        assert_eq!(res.status(), Status::Ok);

        let res = client.get(format!("/repairs/repair/{}", "62ec72bc72a64b75c8719eaf").to_string()).await;
        assert_eq!(res.status(), Status::NotFound);

        let res = client.get(format!("/repairs/repair/{}", 1).to_string()).await;
        assert_eq!(res.status(), Status::Ok);
        let reps_col = db.db.collection::<Repair>("repairs");
        _ = reps_col.delete_one(doc!{"_id": rep_id}, None).await;

         
    }
    #[async_test]
    async fn test_get_catalog_repair() {
        let mut db = DbFixture::new().await;
        let mut client = LoggedClient::init().await;
        let user_id: String = client.with_user("test_update_repair", &mut db, Some("Admin".to_string())).await;
        let user_id = ObjectId::from_str(&user_id).unwrap();
        let technician = ObjectId::new();
        let log_entry = Log{ entry: "Recibida".to_string(), status: RepairState::Received, created_at: Utc::now(), by: "Someone".to_string()};
        let res = create_dummy_repair(&technician, &db.db, "cellphone".to_string(),"Recibida".to_string(), &user_id, log_entry).await;
        let rep_id = res.0;  
        let cus_id = res.1;
        // let res = client.get(format!("/repairs/catalog?technician={}&repair_state=Recibida&est_fix_date=2022-08-05", "62edad7ee8168d33191cf13b").to_string()).await;
        let res = client.get("/repairs/catalog?repair_state=Received&repair_state=Voided&sort_field=estimatedFixedDate".to_string()).await;
        // assert_eq!(res.status(), Status::Ok);
        dbg!(&res.into_string().await);
    }
    // #[async_test]
    // async fn test_get_catalog_repair() {
    //     let mut db = DbFixture::new().await;
    //     let mut client = LoggedClient::init().await;
    //     let user_id: String = client.with_user("test_update_repair", &mut db, Some("Admin".to_string())).await;
    //     let user_id = ObjectId::from_str(&user_id).unwrap();
    //     let technician = ObjectId::new();
    //     let log_entry = Log{ entry: "Recibida".to_string(), status: RepairState::Received, created_at: Utc::now(), by: "Someone".to_string()};
    //     let res = create_dummy_repair(&technician, &db.db, "cellphone".to_string(),"Recibida".to_string(), &user_id, log_entry).await;
    //     let rep_id = res.0;  
    //     let cus_id = res.1;
    //     // let res = client.get(format!("/repairs/catalog?technician={}&repair_state=Recibida&est_fix_date=2022-08-05", "62edad7ee8168d33191cf13b").to_string()).await;
    //     let res = client.get("/repairs/catalog?repair_state=Received&repair_state=Voided&sort_field=estimatedFixedDate".to_string()).await;
    //     // assert_eq!(res.status(), Status::Ok);
    //     dbg!(&res.into_string().await);
    // }

    #[async_test]
    async fn test_get_product_types() {
        let mut db = DbFixture::new().await;
        let mut client = LoggedClient::init().await;
        let user_id: String = client.with_user("test_update_repair", &mut db, Some("Admin".to_string())).await;
        let user_id = ObjectId::from_str(&user_id).unwrap();
        let technician = ObjectId::new();
        let log_entry = Log{ entry: "Recibida".to_string(), status: RepairState::Received, created_at: Utc::now(), by: "Someone".to_string()};
        let res = create_dummy_repair(&technician, &db.db, "cellphone".to_string(),"Recibida".to_string(), &user_id, log_entry).await;
        let res = client.get("/repairs/product_types".to_string()).await;
        assert_eq!(res.status(), Status::Ok);
        

    }

}   