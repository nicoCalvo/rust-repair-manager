#![allow(dead_code)]
#![allow(unused_imports)]
use rocket::{Rocket, State, Build};
use rocket::fairing::AdHoc;


mod common;

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use bson::{Document, doc, oid::ObjectId};
    use chrono::Utc;
    use mongodb::Collection;
    use repair_manager::models::repair::{self, Repair};
    use rocket::{tokio::{self, io::AsyncReadExt}, http::ContentType, time::OffsetDateTime};
    use ::rocket::{http::Status, async_test};
    use super::Rocket;
    use ::rocket::local::asynchronous::Client;
    use crate::common::{DbFixture, CustomerBuilder, LoggedClient};


    #[async_test]
    async fn test_create_repair_new_customer() {
        let mut db = DbFixture::new().await;
        let mut client = LoggedClient::init().await;
        client.with_user("test_create_repair_new_cus", &mut db, Some("admin".to_string())).await;
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

        let cus_col = db.db.collection::<Document>("customers");
        _ = cus_col.delete_one(doc!{"_id": customer_id}, None).await;
        _ = repairs_col.delete_one(doc!{"_id": repair_id}, None).await;
        


    }

    // #[async_test]
    // async fn test_create_repair_new_customer() {
    //     todo!()
    // }

    // #[async_test]
    // async fn test_create_repair_invalid_customer() {
    //     todo!()
    // }

}