#![allow(dead_code)]
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
    use repair_manager::models::repair::{self, Repair, RepairState};
    use repair_manager::models::customer::{self, Customer};
    use rocket::{tokio::{self, io::AsyncReadExt}, http::ContentType, time::OffsetDateTime};
    use ::rocket::{http::Status, async_test};
    use serde::{Serialize, Deserialize};
    use super::Rocket;
    use ::rocket::local::asynchronous::Client;
    use crate::common::{DbFixture, CustomerBuilder, LoggedClient, create_dummy_repair};

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Repair2 {
        #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
        pub id: Option<bson::oid::ObjectId>,
        pub received_by: String,
        pub received_by_id: ObjectId,
        pub customer: bson::oid::ObjectId,
        // pub product: RepairedProduct,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub technician: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub technician_id: Option<ObjectId>,
        // pub logs: Vec<Log>,
        pub status: String,
        pub  description: String,
        pub additional: String,
        pub suggested_price: i32,
        pub warranty: i16,
        // pub received_date: bson::DateTime,
        pub received_date: chrono::DateTime<chrono::Utc>,
        // #[serde(with = "date_format")]
        // pub estimated_fixed_date: chrono::NaiveDate,
        pub finished_repair: Option<chrono::DateTime<chrono::Utc>>,
        pub delivered_date: Option<chrono::DateTime<chrono::Utc>>,
        pub voided_date: Option<chrono::DateTime<chrono::Utc>>,
        // pub bill: Option<Bill>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub billed_by_id: Option<ObjectId>,
        pub billed_by: Option<String>,
        pub voided: bool,
        pub repair_id: i32 // rename from old_id in migration project
        
    }

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
                "street": "avenida siempre viva",
                "number": "2",
                "phone": "12345",
                "email": "si"
            },
            "product":{
                "product_type": "cellphone",
                "brand": "Samsung",
                "model": "asd-123",
                "serial_number": "123-dfsdfds"
            },
            "description": "No le anda",
            "additional": "si",
            "suggested_price": 23,
            "warranty": 6,
            "estimated_fixed_date": Utc::now().date().format("%Y-%m-%d").to_string()
        };
        
        let res = client.post(&repair_request, "/repairs/repair".to_string()).await;
        assert_eq!(res.status(), Status::Ok);
        let res = res.into_json::<Document>().await.unwrap();
        let repair_id = res.get_object_id("repair_id").unwrap();
        let customer_id = res.get_object_id("customer_id").unwrap();
        // aca validar que la repair exista y luego borrarla
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
                "street": "avenida siempre viva",
                "number": "2",
                "phone": "12345",
                "email": "si"
            },
            "product":{
                "product_type": "cellphone",
                "brand": "Samsung",
                "model": "asd-123",
                "serial_number": "123-dfsdfds"
            },
            "description": "No le anda",
            "additional": "si",
            "suggested_price": 23,
            "warranty": 6,
            "estimated_fixed_date": Utc::now().date().format("%Y-%m-%d").to_string()
        };
        
        let res = client.post(&repair_request, "/repairs/repair".to_string()).await;
        assert_eq!(res.status(), Status::Ok);
        let res = res.into_json::<Document>().await.unwrap();
        let repair_id = res.get_object_id("repair_id").unwrap();
        let customer_id = res.get_object_id("customer_id").unwrap();
        // aca validar que la repair exista y luego borrarla
        let repair_request = doc!{
            "customer":{
                "name": "test_create_repair_existing_customer_new_product",
                "last_name": "some_last_name",
                "location": "white bay",
                "street": "avenida siempre viva",
                "number": "2",
                "phone": "12345",
                "email": "si"
            },
            "product":{
                "product_type": "cellphone",
                "brand": "Samsung",
                "model": "asd-123",
                "serial_number": "123-dfsdfds-XXX"
            },
            "description": "No le anda",
            "additional": "si",
            "suggested_price": 23,
            "warranty": 6,
            "estimated_fixed_date": Utc::now().date().format("%Y-%m-%d").to_string()
        };

        let res = client.post(&repair_request, "/repairs/repair".to_string()).await;
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
                "street": "avenida siempre viva",
                "number": "2",
                "phone": "12345",
                "email": "si"
            },
            "product":{
                "product_type": "cellphone",
                "brand": "Samsung",
                "model": "asd-123",
                "serial_number": "123-dfsdfds"
            },
            "description": "No le anda",
            "additional": "si",
            "suggested_price": 23,
            "warranty": 6,
            "estimated_fixed_date": Utc::now().date().format("%Y-%m-%d").to_string()
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
                "street": "avenida siempre viva",
                "number": "2",
                "phone": "12345",
                "email": "si"
            },
            "product":{
                "product_type": "cellphone",
                "brand": "Samsung",
                "model": "asd-123",
                "serial_number": "123-dfsdfds-XXX"
            },
            "description": "No le anda",
            "additional": "si",
            "suggested_price": 23,
            "warranty": 6,
            "estimated_fixed_date": Utc::now().date().format("%Y-%m-%d").to_string()
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
        let user_id: String = client.with_user("test_create_repair_new_cus", &mut db, Some("Admin".to_string())).await;
        let user_id = ObjectId::from_str(&user_id).unwrap();
        let res = create_dummy_repair(&user_id, &db.db, "cellphone".to_string(),"Recibida".to_string(), &user_id).await;
        let rep_id = res.0;  
        let cus_id = res.1;
        let repair_request = doc!{
            "repair_id": rep_id,
            "status": "En progreso",
        };
        let res = client.put(&repair_request, "/repairs/repair".to_string()).await;
        // assert_eq!(res.status(), Status::UnprocessableEntity);
        dbg!(res.into_string().await);
    }
}