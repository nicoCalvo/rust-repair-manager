#![allow(dead_code, unused_imports)]
use crate::models;

use crate::database;
use crate::models::repaired_product;
use crate::utils;

// use models::repaired_product;
// mod models;

use bson::oid::ObjectId;
use bson::to_document;
use chrono::Utc;
// use models::repaired_product::RepairedProduct;
use models::repaired_product::RepairedProduct;
use mongodb::Collection;
use mongodb::Database;
use mongodb::bson::{Document, doc};
use mongodb::options::{FindOptions, CountOptions};
use rocket::response::status::Forbidden;
use utils::date_format;

use serde::{Deserialize, Serialize};
use rocket::serde::json::Json;
use rocket::{State, Response};

use database::db::DbPool;
use models::customer::Customer;
use super::request_guards::user::UserRequest;

/*

1 - Endpoint de get repairs que acepte:

 - fecha
 - estado
 - filtro por tecnico
 - limite

 - ordenar por:
    - fecha de entrega


2- Endpoint de process repair:
   
    - Agrega una entrada de log opcional y cambia el estado de la reparacion
    - todos los estados


3- Endpoint de close repair:

   - Acepta un monto
   - cambia el estado de la reparacion
   - Solo se puede hacer si esta "to be delivered"


4 - Endpoint de create repair:

   - Dict con cliente
   - Dict con producto
   - Dict de la reparacion en si

*/
#[derive(Responder)]
pub enum ApiError {
    #[response(status=403)]
    Forbidden(String),
    #[response(status = 422)]
    UnprocesableEntity(String),
    #[response(status = 500)]
    InternalError(String)
}



#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CustomerRequest{
    pub name: String,
    pub last_name: String,
    pub location: String,
    pub street: String,
    pub number: String,
    pub phone: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct RepairRequest{
    pub customer: CustomerRequest,
    pub product: RepairedProduct,
    pub description: String,
    pub additional: Option<String>,
    pub suggested_price: i32,
    #[serde(with = "date_format")]
    pub estimated_fixed_date: chrono::NaiveDate,
}



#[post("/repair", format = "json", data = "<repair_request>")]
pub async fn create_repair(
    mut repair_request: Json<RepairRequest>,
    user: UserRequest,
    db: &State<DbPool>
)-> Result<Json<Document>, ApiError>{
 
    let asd = Utc::now().date().naive_utc();
    if repair_request.estimated_fixed_date < asd{
        return Err(ApiError::Forbidden("Invalid est fixed date".to_string()))
    }
    let customers_col = db.mongo.collection::<Customer>("customers");

    let mut customer: Customer = match create_or_restore_customer(&customers_col, repair_request.customer.clone()).await{
        Ok(cus)=> cus,
        Err(_e)=>{
            println!("error!");
            return Err(ApiError::InternalError("unable to restore or create customer".to_string()))
        }
    };
    let product: &RepairedProduct = match create_or_restore_product(&mut customer, &customers_col, &mut repair_request.product).await{
        Ok(prod) => prod,
        Err(_e)=>{
            return Err(ApiError::InternalError("unable to restore or create customer".to_string()))
        }
    };

    // now with the customer id and product detail, proceed to find incremental id for next repair
    //


    Ok(Json(doc!{"customer_id": customer.id.unwrap().to_hex()}))
   
}


async fn create_or_restore_customer(customer_col: &Collection<Customer>, customer_data: CustomerRequest )
-> Result<Customer, mongodb::error::Error>{

    let mut _filter = doc!{
        "name": customer_data.name,
        "last_name": customer_data.last_name,
        "location": customer_data.location,
        "street": customer_data.street,
        "number": customer_data.number,
        "phone": customer_data.phone,
    };
    
    let res = customer_col.find_one(_filter.clone(), None).await?;
    if let Some(cus) = res {
        println!("############CUSTOMER FOUND!!");
        Ok(cus)
    }else{
        _filter.extend(doc!{"repaired_products": []});
        let mut customer: Customer = bson::from_bson::<Customer>(bson::to_bson(&_filter).unwrap()).unwrap();
        let res = customer_col.insert_one(&customer, None).await?;
        customer.id = res.inserted_id.as_object_id();
        Ok(customer)
    }
   
}


async fn create_or_restore_product<'a>(
    customer: &'a mut Customer,
    customers_col: &Collection<Customer>,
    repaired_product: &'a mut RepairedProduct
)-> Result<&'a RepairedProduct, mongodb::error::Error>{
    let product_res = customer.repaired_products.iter()
    .find(|prod|{
        prod.brand == repaired_product.brand && prod.product_type == repaired_product.product_type &&
        prod.model == repaired_product.model && prod.serial_number == repaired_product.serial_number
    });
    // return existing product if so  or create a new one and push it to customer's product list
    if let Some(prod) = product_res{
        Ok(prod)
    }else{
        repaired_product.id = Some(ObjectId::new());
        let update_query = doc!{
            "$push": {
                "repaired_products": to_document(&repaired_product).unwrap()
            }
        };
        let res = customers_col.update_one(doc!{"_id": customer.id.unwrap()}, update_query, None).await?;
        return Ok(repaired_product);
    }
}

fn create_repair_new_customer(customer: Customer, db: Database){
    todo!()
}

fn create_repair_existing_customer(){}
// #[post("/create", format = "json", data = "<repair_request>")]
// pub async fn create_repair(
//     repair_request: Json<Customer>,
//     cookies: &CookieJar<'_>,
//     mongo_db: &State<DbPool>
// ) {
//     // hacer la prueba de crear un customer sin _id a ver si lo crea
//     let col_cus: Collection<User> = mongo_db.mongo.collection("customers");
//     // let customer_doc = 
//     let customer_doc = bson::to_bson(&repair_request).unwrap();
//     match col_cus.insert_one(customer_doc, None) {
//         Ok(res) => println!("Customer inserted piola: {:?}", res),
//         Err(e) => println!("ERROR! {:?}", e)
//     }

// }

