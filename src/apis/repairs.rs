#![allow(dead_code)]
#[allow(unused_imports)]
use crate::models;

use crate::database;

// use models::repaired_product;
// mod models;

// use models::repaired_product::RepairedProduct;
use models::repaired_product::RepairedProduct;
use mongodb::Collection;
use mongodb::bson::{Document, doc};
use mongodb::options::{FindOptions, CountOptions};
use serde::{Deserialize, Serialize};
use rocket::http::{Cookie, CookieJar, ContentType};
use rocket::serde::json::Json;
use rocket::{State, Response};

use models::customer::Customer;

use database::db::DbPool;

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



// #[derive(Serialize, Deserialize, Debug)]
// pub struct CustomerRequest{
//     pub _id: Option<String>,
//     pub old_id: Option<i32>,
//     pub name: String,
//     pub last_name: String,
//     pub location: String,
//     pub  street: String,
//     pub number: String,
//     #[serde(skip_serializing_if = "Option::is_none")]
//     pub phone: Option<String>,
//     #[serde(skip_serializing_if = "Option::is_none")]
//     pub email: Option<String>,
// }



// #[derive(Serialize, Deserialize, Debug)]
// pub struct RepairRequest{
//     repair: String,
//     customer: CustomerRequest,
//     product: RepairProductRequet
// }


#[post("/create")]
pub async fn create_repair(){
    todo!()
}

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

