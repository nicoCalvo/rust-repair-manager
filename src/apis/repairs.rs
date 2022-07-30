#![allow(dead_code, unused_imports)]
use std::str::FromStr;

use crate::models;

use crate::database;
use crate::models::repair::{Repair, RepairState};
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
use models::user::Role;
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

struct RepairProcessor{}
trait ProcessRepair {
    fn to_status(repair: &mut Repair, requested_state: &RepairState, user: &UserRequest)-> bool{
        match repair.status{
            RepairState::Received => _process_received(repair, requested_state),
            RepairState::InProgress => _process_in_progress(repair, requested_state, user),
            RepairState::Delivered => _process_delivered(repair, requested_state),
            RepairState::Voided => _process_voided(repair, requested_state),
            RepairState::Budget => _process_voided(repair, requested_state),
            RepairState::Derived => _process_voided(repair, requested_state),
            RepairState::Repaired => _process_voided(repair, requested_state),
            RepairState::NotRepaired => _process_voided(repair, requested_state)
        }
    }
}

impl ProcessRepair for RepairProcessor{}


fn _process_received(repair: &mut Repair, requested_state: &RepairState)-> bool{
    todo!()
}
fn _process_in_progress(repair: &Repair, requested_state: &RepairState, user: &UserRequest)-> bool{
    // re assignment can only be done by an admin
    // only current technician can add entries to logs and change status (or admin)
    let original_status = &repair.status;
    if !matches!(user.role, Role::Admin) && user.id != repair.technician_id.unwrap() {
        println!("User {} attempted to change repair assigned to: {}", user.name, repair.technician.as_ref().unwrap());
        return false
    };
    if ![RepairState::Budget, RepairState::Derived, RepairState::Repaired, RepairState::NotRepaired].contains(requested_state){
        let state: String = requested_state.into();
        
       
        let repair_status: String = original_status.into();
        println!("Invalid state requested: {} for in {:?} repair", repair_status, state);
        return false;
    };
    if matches!(RepairState::InProgress, requested_state) && matches!(original_status, requested_state){
        // only log entry?
        // aca


    }
    return true
    /*
        IN PROGRESS a BUDGET
        IN PROGRESS a DERIVED
        IN PROGRESS a REPAIRED
        IN PROGRESS a NOT_REPAIRED

        else is invalid

        need repair_request to obtain expected field
        // repair_request could be change into Document
        and leave the conversion from doc to a struct based on the desired state?
    */
}
fn _process_delivered(repair: &Repair, requested_state: &RepairState)-> bool{
    false
}
fn _process_voided(repair: &Repair, requested_state: &RepairState)-> bool{
    false
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
    let customer_id: ObjectId = customer.id.unwrap();
    let mut product_request = repair_request.product.clone();
    let product: &RepairedProduct = match create_or_restore_product(&mut customer, &customers_col, &mut product_request).await{
        Ok(prod) => prod,
        Err(_e)=>{
            return Err(ApiError::InternalError("unable to restore or create customer".to_string()))
        }
    };
   
    let _filter = doc!{
            "$and":[
                {"customer": {"$eq":customer_id}},
                {"product._id": {"$eq": product.id.unwrap()}},
                {"status": {"$nin": ["Entregada"]}}
            
            ]
        };
    match repairs_col.find_one(_filter, None).await{
        Ok(res) =>{
            if let Some(existing_repair) = res{
                let msg: String = format!("Product is currently under repair: {}", existing_repair.id.unwrap().to_hex());
                return Err(ApiError::UnprocesableEntity(msg))
            }
        },
        Err(_e)=> return Err(ApiError::InternalError("unable to create repair".to_string()))
    };
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
            received_by_id: user.id,
            customer: customer_id,
            product: product.to_owned(),
            technician: None,
            technician_id: None,
            logs: Vec::new(),
            status: RepairState::Received,
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
            billed_by_id: None,
            billed_by: None,
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
        }
    }
    Err(anyhow::anyhow!("Unable to reate repair"))
        
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
    let mut repair_id: i32 = 0; // default if first repair ever
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
        println!("Creating repair for existing customer");
        Ok(cus)
    }else{
        println!("Creating repair for new customer");
        _filter.extend(doc!{"repaired_products": []});
        let mut customer: Customer = bson::from_bson::<Customer>(bson::to_bson(&_filter).unwrap()).unwrap();
        let res = customer_col.insert_one(&customer, None).await?;
        customer.id = res.inserted_id.as_object_id();
        Ok(customer)
    }
   
}


async fn create_or_restore_product<'a>(
    customer: &'a mut Customer,
    customers_col: &'a Collection<Customer>,
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
        println!("NEW PRODUCT FOR CUSTOMER");
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


// ver que campos son opcionales
// se agregan lineas de log de a una


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BillRequest{
    pub amount: i32,
    pub pay_method: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Status{
    Received,
    InProgress,
    Delivered
}

impl FromStr for Status {

    type Err = ();

    fn from_str(input: &str) -> Result<Status, Self::Err> {
        match input {
            "Recibida"  => Ok(Status::Received),
            "En Progreso"  => Ok(Status::InProgress),
            "Entregda"  => Ok(Status::Delivered),
            _      => Err(()),
        }
    }
}



#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateRepairRequest{
    pub repair_id: ObjectId,
    pub warranty: Option<i16>, //default 6 months
    pub suggested_price: Option<i32>,
    pub status: String,
    pub bill: Option<String>,
    pub log_entry: Option<String>
}


#[put("/repair", format = "json", data = "<repair_request>")]
pub async fn update_repair(
    repair_request: Json<UpdateRepairRequest>,
    user: UserRequest,
    db: &State<DbPool>
)-> Result<Json<Document>, ApiError>{
    
    let update_repair = repair_request.into_inner();
    let status = RepairState::from(update_repair.status.as_str());

    dbg!(&update_repair);
   
    let reps_col = db.mongo.collection::<Repair>("repairs");
    let repair = match reps_col.find_one(doc!{"_id": update_repair.repair_id}, None).await{
        Ok(res)=>res,
        Err(_e)=> {
            dbg!(_e);
            return Err(ApiError::InternalError("unable to restore repair".to_string()))
        }
    };
    if repair.is_none(){
        return Err(ApiError::UnprocesableEntity("Repair does not exists".to_string()))
    }
   
    let mut repair = repair.unwrap();
    if RepairProcessor::to_status(&mut repair, &status, &user){
        let lol = to_document(&repair).unwrap();
        return Ok(Json(lol));
    }
    // si el estado es el mismo, solo se admite uan entrada de log
    // si el estado es distinto, hacer las validaciones de cambio de estado
    // si el
    // check if voided:
    dbg!(update_repair);
    if repair.voided{
        return Err(ApiError::UnprocesableEntity("Voided repair is inmutable".to_string()))
    }
   // hacer una funcion en base al estado actual de la 
    Ok(Json(doc!{}))
    /* in progress 
        allowed transitions:
            - esperando presupuesto
            - 

            STATUS_CHOICES = [(RECEIVED, 'Recibida'),
            (IN_PROGRESS, 'En progreso'),
            (TO_BE_DELIVERED, 'Para entregar'),
            (BUDGET, 'Confirmacion presupuesto'),
            (NOT_REPAIRED, 'Sin reparar'),
            (REPAIRED, 'Esperando repuesto'),
            (DELIVERED, 'Entregada'),
            (DERIVED, 'Derivada'),
            ]
        
            hacer metodos en base al estado actual:

        process_in_progress => incluye estado actua BUDGET, IN PROGRESS, DERIVED
            transiciones validas:
                
               
                DERIVED a IN PROGRESS
                BUDGET a IN PROGRESS

        process_received =>
            RECEIVED a IN PROGRESS
        process_to_be_delivered=>
            TO_BE_DELIVERED a DELIVERED

    */
    
    // let FINISHED_STATUS = ['a','b'];
    // if let Some(new_status) = repair_request.status{
    //     if FINISHED_STATUS.contains(repair.status) && new_status{
    //         Err(ApiError::UnprocesableEntity("Repair already finished".to_string()))
    //     }
    //     if new_status == "Para entregar" && FINISHED_STATUS.contains(repair.status){
    //         repair.status = "Para entregar".to_string();
    //         repair.finished_repair = DateTime::now();

    //     }
    // }else{

    // }
        
}
// async fn _update_repair(repair, update_repair, reps_col, db)->{
//     todo!()
// }
/*
update:
    Cambios de estado:
        * no se puede volver a recibida
        * solo se puede voidear una ya entregada
        * recibida - en progreso ------- Finalizada
                                    |--- Esperando confirmacion presupuesto
                                    |--- algo mas
                                    ---- Derivada
        * No se puede poner precio negativo de precio sugerido

    Finalizacion:
        * agregar fecha de finalizacion
    
    Entrega:
        * solo en estado "para entregar"
        * Agregar Bill (precio positivo y forma de pago)
    
  

    
*/
//     Ok(Json(doc!{"repair_id": repair_request.repair_id}))
// }

//anulacion en metodo delete
/*
Anulacion:
* Solo las entregadas

*/