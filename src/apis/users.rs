// #![allow(unused_imports, dead_code)]
// create-restore-delete(deactivate only)
// admin only

use bson::Document;
use bson::to_document;
use mongodb::bson::doc;
use rocket::State;
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};

use crate::database;
use crate::models;
use crate::utils;

use database::db::DbPool;
use models::user::{User, Role};
use super::{request_guards::user::AdminRequest};
use utils::hash_password;

#[derive(Serialize, Deserialize)]
pub struct UserId{
    _id: String
}


#[derive(Responder)]
pub enum ApiError {
    #[response(status = 422)]
    UnprocesableEntity(String),
    #[response(status = 500)]
    InternalError(String)
}

#[derive(Serialize, Deserialize)]
pub struct UserRequest{
    username: String,
    email: String,
    role: Role,
    password: Option<String>
}


#[post("/", format="application/json", data="<post_user>")]
pub async fn create_user(
    post_user: Json<UserRequest>,
    _admin_req: AdminRequest,
    db: &State<DbPool>
) -> Result<Json<UserId>, ApiError>{
    // check user does not exists already:
    
    let mut post_user = post_user.into_inner();
    post_user.email = post_user.email.to_lowercase();
    let users_col = db.mongo.collection::<User>("users");
    let _filter = doc!{
        "$and":[
                  {"username": {"$eq": post_user.username.clone().to_lowercase()}},
                  {"email": {"$eq": post_user.email.clone()}}
                 
              ]   
        };
    let user_res = users_col.find_one(_filter, None).await.unwrap();
    if let Some(user) = user_res {
        let id = user.id.unwrap().to_hex();
        return Err(ApiError::UnprocesableEntity(format!("User already exists {}" , id)))
    }
    let mail_name_iter = post_user.email.split("@");
    let mail_name_vec: Vec<&str> = mail_name_iter.collect();
    let mut password = format!("{}123", mail_name_vec[0]);
    password = hash_password(&password);
    post_user.password = Some(password);
    let user = User::from(to_document(&post_user).unwrap());
    match users_col.insert_one(user, None).await{
        Ok(id) =>  return Ok(Json(UserId{_id: id.inserted_id.as_object_id().unwrap().to_hex()})),
        Err(_e)=>  return Err(ApiError::InternalError("Something really awful happened!".to_string()))
    }

}



#[get("/")]
pub async fn get_users(
    _admin_req: AdminRequest,
    db: &State<DbPool>
) -> Result<Json<Vec<Document>>, ApiError>{
    let users_col = db.mongo.collection::<User>("users");
    let mut users_res: Vec<Document> = Vec::new();
    let res = users_col.aggregate([doc!{ "$project":{"password": 0} }], None).await;
 
    if let Ok(mut cursor) = res {
        while cursor.advance().await.unwrap_or(false){
            let user = cursor.deserialize_current().unwrap();
            users_res.push(user);

        }
        Ok(Json(users_res))
    } else {
        return Err(ApiError::InternalError("Something really awful happened!".to_string()))
    }
    

}





#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserObjID{
    #[serde(skip_serializing)]
    pub ids: Vec<bson::oid::ObjectId>
}


#[put("/activate", format="application/json", data="<user_obj_id>")]
pub async fn activate_user(
    _admin_req: AdminRequest,
    user_obj_id: Json<UserObjID>,
    db: &State<DbPool>
) -> Result<(), ApiError>{
    let users_col = db.mongo.collection::<User>("users");

    let res = users_col.find(doc!{"_id": {"$in": user_obj_id.ids.clone()}}, None).await;
    match res {
        Ok(mut cursor) => {
            while cursor.advance().await.unwrap(){
                let user: User = cursor.deserialize_current().unwrap();
                info!("changing active status for user {:?} to: {}", user.id, !user.active);
                let query = doc!{"$set": {"active": !user.active}};
                let res = users_col.update_one(doc!{"_id": user.id}, query, None).await;
                if res.is_err(){
                    error!("unable to update user active status: {}", res.err().unwrap());
                    return Err(ApiError::InternalError("Something really awful happened!".to_string()))
                };

            }
        },
       
        Err(e) =>{
            error!("Unable to restore user ids: {:?} - ERROR: {}", user_obj_id.ids, e.to_string());
            return Err(ApiError::InternalError("Unable to perform the operation".to_string()))
        }
    };
    Ok(())


}
