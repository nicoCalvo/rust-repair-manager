#![allow(dead_code, unused_imports)]
use crate::models;

use crate::database;
use crate::models::repair::Repair;
use crate::models::repaired_product;
use crate::utils;

// use models::repaired_product;
// mod models;

use bson::oid::ObjectId;
use bson::to_document;
use chrono::NaiveDate;
use chrono::Utc;
// use models::repaired_product::RepairedProduct;
use models::repaired_product::RepairedProduct;
use mongodb::Collection;
use mongodb::Database;
use mongodb::bson::{Document, doc};
use mongodb::error::Error;
use mongodb::error::WriteError;
use mongodb::options::{FindOptions, CountOptions};
use rocket::response::status::Forbidden;
use utils::date_format;
use thiserror::Error;


use serde::{Deserialize, Serialize};
use rocket::serde::json::Json;
use rocket::{State, Response};
use anyhow::Result as AnyResult;

use database::db::DbPool;
use models::customer::Customer;
use super::request_guards::user::UserRequest;

const MAX_RETRIES: i32 = 10;

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
    pub warranty: i16, //default 6 months
    pub suggested_price: i32,
    #[serde(with = "date_format")]
    pub estimated_fixed_date: chrono::NaiveDate,
}



#[post("/repair", format = "json", data = "<repair_request>")]
pub async fn create_repair(
    repair_request: Json<RepairRequest>,
    user: UserRequest,
    db: &State<DbPool>
)-> Result<Json<Document>, ApiError>{
    let repair_request: RepairRequest = repair_request.into_inner();
    let asd = Utc::now().date().naive_utc();
    if repair_request.estimated_fixed_date < asd{
        return Err(ApiError::Forbidden("Invalid est fixed date".to_string()))
    }
    let customers_col = db.mongo.collection::<Customer>("customers");
    let repairs_col = db.mongo.collection::<Repair>("repairs");

    let mut customer: Customer = match create_or_restore_customer(&customers_col, repair_request.customer.clone()).await{
        Ok(cus)=> cus,
        Err(_e)=>{
            println!("error!");
            return Err(ApiError::InternalError("unable to restore or create customer".to_string()))
        }
    };
    let customer_id = customer.id.unwrap();
    let mut product_request = repair_request.product.clone();
    let product: &RepairedProduct = match create_or_restore_product(&mut customer, &customers_col, &mut product_request).await{
        Ok(prod) => prod,
        Err(_e)=>{
            return Err(ApiError::InternalError("unable to restore or create customer".to_string()))
        }
    };
    // let customer_id=customer.clone();
    match _create_repair(&repairs_col, customer_id, product, &repair_request, &user).await{
        Ok(rep_id) =>{
            println!("repair created!");
            let mut response = rep_id.into_inner();
            response.extend(doc!{"customer_id": customer_id});
            return Ok(Json(response))
        },
        Err(_e)=> return Err(ApiError::InternalError("unable to create repair".to_string()))
    };

   
}

async fn  _create_repair(
    repairs_col: &Collection<Repair>,
    customer_id: ObjectId,
    product: &RepairedProduct,
    repair_request: &RepairRequest,
    user: &UserRequest
)-> AnyResult<Json<Document>, anyhow::Error>{
    let mut created = false;
    let mut tries = 1;
    while !created && tries < MAX_RETRIES{
        let latest_repair: i32 =_get_latest_repair_id(repairs_col).await?;
        let repair = Repair{
            id: None,
            received_by: user.name.to_string(),
            customer: customer_id,
            product: product.to_owned(),
            technician: None,
            logs: Vec::new(),
            status: "Recibida".to_string(),
            description: repair_request.description.to_string(),
            additional: repair_request.additional.to_owned().unwrap(),
            suggested_price: repair_request.suggested_price,
            warranty: repair_request.warranty,
            received_date:  Utc::now(),
            estimated_fixed_date: repair_request.estimated_fixed_date,
            finished_repair: None,
            delivered_date: None,
            voided_date: None,
            bill: None,
            voided: false,
            repair_id: latest_repair + 1, // rename old_id for repair_id
        };
        let repair_id: Option<ObjectId> = match repairs_col.insert_one(repair, None).await{
            Ok(res)=>{
                created = true;
                Some(res.inserted_id.as_object_id().unwrap())
            },
            Err(_err)=>{
                    println!("REPAIR ID ALREADY TAKEN");
                    tries += 1;
                    created = false;
                    None
            }
        };
        if let Some(repair_id) = repair_id{
            return Ok(Json(doc!{"repair_id": &repair_id}));
        }else{};
    }
    Err(anyhow::anyhow!("Unable to reate repair"))
        
}

fn fun_name() {
    ()
}
    


async fn _get_latest_repair_id(repairs_col: &Collection<Repair>)
-> Result<i32, Error >{
    let mut cursor = repairs_col.aggregate([
        doc!{"$project": {"repair_id": 1, "_id":0}
        },
        doc!{
         "$sort": {"repair_id": -1}
        },
        doc!{
            "$limit": 1
        }
        ],
        None)
        .await?;
    let res = cursor.advance().await?;
    let mut repair_id: i32 = 1; // default if first repair ever
    if res{
        repair_id = cursor.current().get_i32("repair_id").unwrap();
    }
    Ok(repair_id)

}

async fn create_or_restore_customer(customer_col: &Collection<Customer>, customer_data: CustomerRequest )
-> Result<Customer, Error>{

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
)-> Result<&'a RepairedProduct, Error>{
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
        let _res = customers_col.update_one(doc!{"_id": customer.id.unwrap()}, update_query, None).await?;
        return Ok(repaired_product);
    }
}



