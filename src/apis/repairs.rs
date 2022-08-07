#![allow(unused_imports, dead_code, unused_variables)]
use crate::models;

use crate::database;
use crate::models::repair::{Repair, RepairState, Log, Bill};
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
use mongodb::bson::{Document, doc};
use mongodb::error::Error;
use rocket::http::uri::Query;
use rocket::request::FromParam;
use utils::date_format;


use serde::{Deserialize, Serialize};
use rocket::serde::json::Json;
use rocket::{State};
use anyhow::Result as AnyResult;

use database::db::DbPool;
use models::customer::Customer;
use super::request_guards::user::UserRequest;

const MAX_RETRIES: i32 = 10;
const FORMAT: &'static str = "%Y-%m-%d";

#[derive(Responder)]
pub enum ApiError {
    #[response(status=403)]
    Forbidden(String),
    #[response(status = 422)]
    UnprocesableEntity(String),
    #[response(status = 500)]
    InternalError(String),
    #[response(status=404)]
    NotFound(String)
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
    pub warranty: i32, //default 6 months
    pub suggested_price: i32,
    #[serde(with = "date_format")]
    pub estimated_fixed_date: chrono::NaiveDate,
}


struct RepairProcessor{}
trait ProcessRepair {
    fn to_status(repair: &mut Repair, requested_state: RepairState, user: &UserRequest, update_repair: &UpdateRepairRequest)-> Result<Document, anyhow::Error>{
        match repair.status{
            RepairState::Received => _process_received(repair, requested_state, user, update_repair),
            RepairState::InProgress => _process_in_progress(repair, requested_state, user, update_repair),
            RepairState::Budget | RepairState::Derived | RepairState::WaitingSpare => _process_during_repair(repair, requested_state),
            RepairState::ToBeDelivered | RepairState::NotRepaired => _process_repaired(repair, requested_state, user, update_repair),
            RepairState::Delivered => _process_delivered(repair, requested_state, user, update_repair),
            RepairState::Voided => {
                error!("Attempt to modify voided repair {} by {}", repair.id.unwrap().to_hex(), user.name);
                return Err(anyhow::anyhow!("Cannot modify voided repair"))
            }
            
        }
    }
}

impl ProcessRepair for RepairProcessor{}


fn _process_received(repair: &Repair, requested_state: RepairState, user: &UserRequest, update_repair: &UpdateRepairRequest)->  Result<Document, anyhow::Error>{
    // can only be moved into In Progress
    if !matches!(requested_state, RepairState::InProgress){
        let state_req_str: String = requested_state.into();
        error!("Repair {} in Received state cannot be move to in  {} state", repair.id.unwrap().to_hex(), state_req_str);
        return Err(anyhow::anyhow!("Invalid status change attempt for repair"));
    };
    let next_state: String = RepairState::InProgress.into();
    let new_log = Log{ entry: "Comienzo de reparacion".to_string(), status: RepairState::InProgress, created_at: Utc::now(), by: user.name.clone()};
    let new_log_as_doc =  to_document(&new_log).unwrap();
    let update_query = doc!{
        "$set": {
            "technician": user.name.clone(),
            "technician_id": user.id,
            "status": next_state
        },
        "$push":{
            "logs": new_log_as_doc
        },
    };
    Ok(update_query)

}
fn _process_repaired(repair: &Repair, _requested_state: RepairState, user: &UserRequest, update_repair: &UpdateRepairRequest)->  Result<Document, anyhow::Error>{
    if !matches!(RepairState::Delivered, _requested_state){
        let repair_status: String = repair.status.clone().into();
        let requested_state_str: String = _requested_state.clone().into();
        error!("Invalid state requested: {} for in {:?} repair", repair_status, requested_state_str);
        return Err(anyhow::anyhow!("Invalid state requested: current: {} - desired: {}", requested_state_str, repair_status));
    }
    if update_repair.bill.is_none(){
        error!("No bill provided for repair {} ", repair.repair_id);
        return Err(anyhow::anyhow!("No Bill provided"));
    }
    let next_state: String = RepairState::Delivered.into();
    let new_log = Log{ entry: "Entregada".to_string(), status: RepairState::Delivered, created_at: Utc::now(), by: user.name.clone()};
    let new_log_as_doc =  to_document(&new_log).unwrap();
   
    let update_bill = update_repair.clone().bill.unwrap();
    let bill = Bill{ amount: update_bill.amount, pay_method: update_bill.pay_method, created_at: Utc::now(), by: user.id };
    
    
    let update_query = doc!{
        "$set": {
            "status": next_state,
            "bill": to_document(&bill).unwrap(),
            "billed_by": user.name.clone(),
            "billed_by_id": user.id,
            "delivered_date": Utc::now().to_string()
        },
        "$push": {"logs": new_log_as_doc},
        
    };
    Ok(update_query)

}
fn _process_in_progress(repair: &Repair, requested_state: RepairState, user: &UserRequest, update_repair: &UpdateRepairRequest)->  Result<Document, anyhow::Error>{
    // re assignment can only be done by an admin
    // only current technician can add entries to logs and change status (or admin)
    let original_status = &repair.status;
    let requested_state_str: String = requested_state.clone().into();
    let repair_status: String = original_status.into();
    let log_entry = update_repair.clone().log_entry.unwrap_or(requested_state_str.clone());
    if !matches!(user.role, Role::Admin) && user.id != repair.technician_id.unwrap() {
        error!("[SEC] User {} attempted to change repair assigned to: {}", user.name, repair.technician.as_ref().unwrap());
        return Err(anyhow::anyhow!("Invalid attempt for repair"));
    };
    if [RepairState::Voided, RepairState::Received, RepairState::Delivered].contains(&requested_state){
        error!("Invalid state requested: {} for in {:?} repair", repair_status, requested_state_str);
        return Err(anyhow::anyhow!("Invalid state requested: current: {} - desired: {}", requested_state_str, repair_status));
    }
    if [RepairState::Budget, RepairState::Derived, RepairState::NotRepaired].contains(&requested_state){
       
        let new_log = Log{ entry: log_entry, status: requested_state, created_at: Utc::now(), by: user.name.clone() };
        let new_log_as_doc =  to_document(&new_log).unwrap();
        return Ok(doc!("$push": {"logs": new_log_as_doc},  "$set":{"status": requested_state_str}))
    }else if matches!(requested_state, RepairState::ToBeDelivered){
        let new_log = Log{ entry: log_entry, status: requested_state, created_at: Utc::now(), by: user.name.clone() };
        let new_log_as_doc =  to_document(&new_log).unwrap();
        return Ok(doc!{
            "$push": {
                "logs": new_log_as_doc
            },
            "$set":{
                "status": requested_state_str,
                "suggested_price": update_repair.suggested_price.unwrap_or(0),
                "finished_repair": Utc::now().to_string(),
                "warranty": update_repair.warranty.unwrap_or(6)
            }
        })
    };


    if !matches!(RepairState::InProgress, requested_state) && !matches!(&repair_status, requested_state){
        return Err(anyhow::anyhow!("Invalid state requested: current: {} - desired: {}", &repair_status, requested_state_str));
    }

    if let Some(logline) = &update_repair.log_entry{
        let new_log = Log{ entry: logline.clone(), status: requested_state, created_at: Utc::now(), by: user.name.clone() };
        let new_log_as_doc =  to_document(&new_log).unwrap();
        return Ok(doc!("$push": {"logs": new_log_as_doc}))

    }else{
        return Err(anyhow::anyhow!("no log was provided for repair: {}", repair.id.unwrap().to_hex()));
    }
}

fn _process_delivered(repair: &Repair, requested_state: RepairState,  user: &UserRequest, update_repair: &UpdateRepairRequest)->  Result<Document, anyhow::Error>{
    // can only be moved into "Voided" status
    let original_status = &repair.status;
    let requested_state_str: String = requested_state.clone().into();
    let repair_status: String = original_status.into();
    if !matches!(requested_state, RepairState::Voided){
        return Err(anyhow::anyhow!("Invalid state requested: current: {} - desired: {}", &repair_status, requested_state_str));
    };
    let log_entry = update_repair.log_entry.as_ref().unwrap_or(&requested_state_str).to_owned();

    let new_log = Log{ entry: log_entry, status: requested_state, created_at: Utc::now(), by: user.name.clone() };
    let new_log_as_doc =  to_document(&new_log).unwrap();
    return Ok(doc!{
        "$push": {
            "logs": new_log_as_doc,
        },
        "$set": {
            "status":requested_state_str.clone(),
            "voided": true,
            "voided_date": Utc::now().date().format("%Y-%m-%d").to_string()

        }
        })

}
fn _process_during_repair(repair: &Repair, requested_state: RepairState)->  Result<Document, anyhow::Error>{
    todo!()
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
                info!("repair {} created for customer {}", res.inserted_id.to_string(), customer_id.to_hex());
                Some(res.inserted_id.as_object_id().unwrap())
            },
            Err(_err)=>{
                    error!("Repair: {} already taken", latest_repair + 1);
                    tries += 1;
                    created = false;
                    None
            }
        };
        if let Some(repair_id) = repair_id{
            return Ok(Json(doc!{"repair_id": &repair_id}));
        }
    }
    error!("max retries trying to pick a repair _id");
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
        info!("Creating repair for existing customer");
        Ok(cus)
    }else{
        info!("Creating repair for new customer");
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
        info!("creating new product {:?} for customer {} ", repaired_product, customer.id.unwrap().to_hex());
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


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BillRequest{
    pub amount: i32,
    pub pay_method: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateRepairRequest{
    pub repair_id: ObjectId,
    pub warranty: Option<i32>, //default 6 months
    pub suggested_price: Option<i32>,
    pub status: String,
    pub bill: Option<BillRequest>,
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
    let reps_col = db.mongo.collection::<Repair>("repairs");
    let repair = match reps_col.find_one(doc!{"_id": update_repair.repair_id}, None).await{
        Ok(res)=>res,
        Err(e)=> {
            error!("{}",e.to_string());
            return Err(ApiError::InternalError("unable to restore repair".to_string()))
        }
    };
    if repair.is_none(){
        error!("Repair {} does not exists", update_repair.repair_id);
        return Err(ApiError::NotFound("Repair does not exists".to_string()))
    }
   
    let mut repair = repair.unwrap();

    if repair.voided{
        return Err(ApiError::UnprocesableEntity("Voided repair is inmutable".to_string()))
    }
    match RepairProcessor::to_status(&mut repair, status, &user, &update_repair){
        Ok(update_query)=> match reps_col.update_one(doc!{"_id": repair.id}, update_query, None).await{
                Ok(_) => return Ok(Json(doc!{"_id": repair.id, "status": update_repair.status.as_str()})),
                Err(e) =>{
                    error!("{}", e);
                    return Err(ApiError::UnprocesableEntity(format!("Unable to update repair")))
                }
        },
        Err(e)=> return Err(ApiError::UnprocesableEntity(format!("Unable to update repair: {:?}", e)))
        
    };
  
        
}



#[get("/repair/<id>", rank = 2)]
pub async fn repair_string(id: String , user: UserRequest, db: &State<DbPool>)-> Result<Json<Document>, ApiError>{
    let repairs_col = db.mongo.collection::<Repair>("repairs");
    let _id =  match ObjectId::parse_str(&id){
        Ok(id) => id,
        Err(_) => return Err(ApiError::NotFound(Default::default()))
    };
    let match_id = doc!{
        "$match":{"_id":_id}
        };
    let customer_info = doc!{
            "$lookup":{
                "from": "customers",
                "localField": "customer",
                "foreignField": "_id",
                "pipeline":[
                    {
                        "$project": {"products": 0}
                    }
                ],
                "as": "customer_info"
                    
            }
        };
    
    match repairs_col.aggregate([match_id, customer_info], None).await{
        Ok(mut cursor) => {
            let res = cursor.advance().await.unwrap();
            if !res{
                info!("Repair {:?} not found", _id);
                return Err(ApiError::NotFound(Default::default()))
            }else{
                let repair = cursor.deserialize_current().unwrap();
                return Ok(Json(repair));
            }
        },
        Err(e)=>{
            error!("Unable to restore repair: {:?}", id);
            return Err(ApiError::UnprocesableEntity(format!("Unable to restore repair: {:?}", e)))
        }
    }
}


#[get("/repair/<id>")]
pub async fn repair_int(id: i32, user: UserRequest, db: &State<DbPool>)-> Result<Json<Document>, ApiError>{
    let repairs_col = db.mongo.collection::<Repair>("repairs");
    let match_id = doc!{
        "$match":{"repair_id": &id}
        };
    let customer_info = doc!{
            "$lookup":{
                "from": "customers",
                "localField": "customer",
                "foreignField": "_id",
                "pipeline":[
                    {
                        "$project": {"products": 0}
                    }
                ],
                "as": "customer_info"
                    
            }
        };
    match repairs_col.aggregate([match_id, customer_info], None).await{
        Ok(mut cursor) => {
            let res = cursor.advance().await.unwrap();
            if !res{
                info!("Repair {:?} not found", &id);
                return Err(ApiError::NotFound(Default::default()))
            }else{
                let repair = cursor.deserialize_current().unwrap();
                return Ok(Json(repair));
            }
        },
        Err(e)=>{
            error!("Unable to restore repair: {:?}", &id);
            return Err(ApiError::UnprocesableEntity(format!("Unable to restore repair: {:?}", e)))
        }
    }
}



#[derive(Debug, PartialEq, FromFormField)]
pub enum VoidedState{
    True,
    False
}


#[derive(Debug, PartialEq, FromFormField)]
pub enum SortField{
    status,
    technician,
    estimatedFixedDate,
    deliveredDate
}
impl ToString for SortField{
    fn to_string(&self) -> String {
        match self{
            SortField::status => "status".to_string(),
            SortField::technician => "technician".to_string(),
            SortField::estimatedFixedDate => "estimated_fixed_date".to_string(),
            SortField::deliveredDate => "delivered_date".to_string(),
        }
    }
}


#[derive(Debug, PartialEq, FromFormField)]
pub enum SortOrd{
    asc,
    desc,
}


#[get("/catalog?<technician>&<repair_state>&<est_fix_date>&<received_date>&<voided>&<sort_field>&<sort_ord>")]
pub async fn catalog(
    technician: Option<&str>,
    repair_state: Vec<RepairState>,
    est_fix_date: Option<&str>,
    received_date: Option<&str>,
    voided: Option<VoidedState>,
    sort_field: Option<SortField>,
    sort_ord: Option<SortOrd>,
    user: UserRequest,
    db: &State<DbPool>
) -> Result<Json<Vec<Document>>, ApiError>{
    let repairs_col = db.mongo.collection::<Repair>("repairs");

    let repair_state_filter = doc!{"status": {"$in": repair_state.into_iter().map(move |s|->String{RepairState::into(s)}).collect::<Vec<String>>()}};
    let technician_filter = match technician{
        Some(tech)=> doc!{"technician": {"$eq": tech}},
        None => doc!{}
    };
    let today =Utc::now().date().to_string();
    let mut parsed_est_fixed_date_filter = doc!{};
    if let Some(est_fix_date) = est_fix_date{
        parsed_est_fixed_date_filter = match NaiveDate::parse_from_str(&est_fix_date, FORMAT){
            Ok(parsed) => doc!{"estimated_fixed_date": {"$gte": est_fix_date.to_string()}},
            Err(e)=> return Err(ApiError::UnprocesableEntity(format!("Invalid estimated fixed date format: {:?}", e)))
        };
    };
    let mut parsed_received_date = doc!{};
    if let Some(rec_date) = received_date{
        parsed_received_date = match NaiveDate::parse_from_str(rec_date, "%Y-%m-%d"){
            Ok(date) => doc!{"received_date": {"$gte": rec_date}},
            Err(e) => return Err(ApiError::UnprocesableEntity(format!("Invalid date format: {:?}", e)))
        };
    }
    // let voided_filter = doc!{"voided": voided.is_some() && voided.unwrap() == VoidedState::True};
    let filter_condition = doc!{
        "$and":[
            technician_filter,
            repair_state_filter,
            parsed_est_fixed_date_filter,
            parsed_received_date,
            // voided_filter
        ]
    };
    let sort_condition = match sort_field{
        Some(field)=>{
            doc!{
                "$sort": {field.to_string(): 1}
            }
        },
        None => doc!{}
           
    };
    let customer_info = doc!{
            "$lookup":{
                "from": "customers",
                "localField": "customer",
                "foreignField": "_id",
                "pipeline":[
                    {
                        "$project": {"products": 0}
                    }
                ],
                "as": "customer_info"
                    
            }
        };
    // let project = doc!{"$project": {"status": 1, "_id": 1}};
    match repairs_col.aggregate([ doc!{"$match": filter_condition}, customer_info ], None).await{
        Ok(mut cursor) => {
            let mut results: Vec<Document> = Vec::new();
            while cursor.advance().await.unwrap(){
                results.push(cursor.deserialize_current().unwrap())
            }
            return Ok(Json(results));
        },
        Err(e)=>{
            error!("Unable to restore repairs");
            return Err(ApiError::UnprocesableEntity(format!("Unable to restore repairs: {:?}", e)))
        }
    }
}
