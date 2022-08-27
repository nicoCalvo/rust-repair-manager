use bson::Document;
use bson::oid::ObjectId;
use bson::to_document;
use mongodb::bson::doc;
use rocket::State;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

use crate::database;
use crate::models;

use database::db::DbPool;
use models::customer::Customer;
use super::{request_guards::user::UserRequest};



#[derive(Serialize, Deserialize)]
pub struct CustomerId{
    _id: String
}


#[derive(Responder)]
pub enum ApiError {
    #[response(status = 422)]
    UnprocesableEntity(String),
    #[response(status = 500)]
    InternalError(String)
}

#[post("/", format="application/json", data="<post_customer>")]
pub async fn create_customer(
    post_customer: Json<Customer>,
    _user_req: UserRequest,
    db: &State<DbPool>
) -> Result<Json<CustomerId>, ApiError>{
    // check if customer exists based on name, lastname and address and location
    let post_customer = post_customer.into_inner();
    
    let customers_col = db.mongo.collection::<Customer>("customers");
    let _filter = doc!{
        "$and":[
                  {"name": {"$eq": post_customer.name.clone()}},
                  {"last_name": {"$eq": post_customer.last_name.clone()}},
                  {"location": {"$eq": post_customer.location.clone()}},
                  {"address": {"$eq": post_customer.address.clone()}},
                 
              ]   
        };
    // cannot apply if collection has been tipified as Customer
    // let opt = FindOneOptions::builder().projection(doc!{"_id": 1}).build();
  
    let customer = customers_col.find_one(_filter, None).await;
    match customer {
        Ok(c) =>{
            let mut _id = String::new();
            if let Some(cus) = c{
                _id = cus.id.unwrap().to_hex();
                return Err(ApiError::UnprocesableEntity(format!("Customer already exists {}" , _id)))
            }else {
                let result = customers_col.insert_one(post_customer, None).await.unwrap();
                _id = result.inserted_id.as_object_id().unwrap().to_hex();
            }
            return Ok(Json(CustomerId{_id}));
        },
        Err(_e) =>{
            return Err(ApiError::InternalError("Unable to retrieve customer".to_string()))

        }
    }
    // Ok(Json(custo))
}



#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateCustomer {
    #[serde(skip_serializing)]
    pub id: bson::oid::ObjectId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}



#[put("/", format="application/json", data="<update_customer>")]
pub async fn update_customer(
    update_customer: Json<UpdateCustomer>,
    _user_req: UserRequest,
    db: &State<DbPool>
) -> Result<Json<CustomerId>, ApiError>{
    let customer = update_customer.into_inner();
    let customers_col = db.mongo.collection::<Customer>("customers");
    let doc = doc!{
        "$set": 
            to_document(&customer).unwrap()
    };
    dbg!(&doc);
    println!("DOC!:{}", customer.id);
    match customers_col.update_one(doc!{"_id": customer.id}, doc, None).await{
        Ok(_) =>{
            println!("Customer {:?} updated", customer.id);
            Ok(Json(CustomerId{_id: customer.id.to_hex()}))
        },
        Err(e) =>{
            println!("ERROR! {}", e);
            Err(ApiError::InternalError("Unable to retrieve customer".to_string()))
        }
    }

}



#[get("/?<id>&<name>&<last_name>")]
pub async fn get_customers(
    id: Option<&str>,
    name: Option<&str>,
    last_name: Option<&str>,
    _user: UserRequest,
    db: &State<DbPool>
)->Result<Json<Vec<Document>>, ApiError>{
    let cus_col = db.mongo.collection::<Document>("customers");
    let id_filter = match id{
        Some(id)=> doc!{"_id": ObjectId::parse_str(id).unwrap()},
        None => doc!{}
    };
    let name_filter = match name{
        Some(name)=> doc!{"name": name},
        None => doc!{}
    };
    let last_name_filter = match last_name{
        Some(last_name)=> doc!{"last_name": last_name},
        None => doc!{}
    };

    let match_query = doc!{
        "$match": {
            "$and": [
                id_filter,
                name_filter,
                last_name_filter
            ]
        }
    };
    match cus_col.aggregate([match_query], None).await{
        Ok(mut cur)=>{
            let mut customers: Vec<Document> = Vec::new();
            while cur.advance().await.unwrap(){
                let customer: Document = cur.deserialize_current().unwrap();
                customers.push(customer)
            };
            return Ok(Json(customers))
        },
        Err(_e)=>{
            Err(ApiError::InternalError("Unable to retrieve customer".to_string()))
        }
    }

}