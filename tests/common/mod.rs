#![allow(dead_code)]
#![allow(unused_imports)]
use std::collections::HashMap;
use std::env;
use std::path::Path;
use bson::oid::ObjectId;
use dotenv::dotenv;
use chrono::prelude::*;
use bson::{Document, Bson};
use mongodb::{Database, Collection, Client, bson::doc};
use repair_manager::models::repair::{Repair, RepairState, Log};
use ::rocket::local::asynchronous::Client as RocketClient;

use repair_manager::models::customer::Customer;
use repair_manager::models::repaired_product::RepairedProduct;
use repair_manager::models::user::User;


use repair_manager::utils::hash_password;
use rocket::http::{Cookie, ContentType, Status};
use rocket::local::asynchronous::LocalResponse;
use rocket::time::{Duration, OffsetDateTime};
use rocket::tokio::fs;
use serde::{Serialize};
use crate::common::Bson::Null;
use rocket::local::asynchronous::Client as RClient;

const FIXTURE_PATH: &str = "tests/fixtures";



pub async fn connect() -> Database {
    dotenv().ok(); 
    let database_url = env::var("MONGO_DB_URL")
        .expect("MONGO_DB_URL must be set");
    let client = Client::with_uri_str(&database_url).await
        .expect(&format!("Unable to connect to DB: {}", database_url));
    client
        .database("admin")
        .run_command(doc! {"ping": 1}, None)
        .await.expect("Unable to reach server");
    println!("Connected successfully.");
    client.database("test_arrobatech")
}


pub fn create_repair(){
    todo!()
}

pub fn create_customer_with_records(){

}

pub struct DbFixture{
    pub db: Database,
    items: HashMap<String, Vec<ObjectId>>

}


async fn get_fixture(fixture_name: String) -> Vec<Document>{
    let path = env::current_dir().unwrap();
    let more_path = path.as_os_str().to_string_lossy().to_string();
    let fixtures_path = Path::new(&more_path);
    let fixture_file_path = fixtures_path.join(Path::new(FIXTURE_PATH));
    let fixture = fixture_file_path.join(Path::new(&format!("{}.json", fixture_name)));
    let asd = fs::read_to_string(fixture.as_os_str().to_str().unwrap()).await.unwrap();
    serde_json::from_str(&asd).expect("JSON was not well-formatted")

}


impl DbFixture{
    pub async fn new() -> Self{
        let db = connect().await;
        Self{
            db: db,
            items: HashMap::new()
        }
     
    }

    pub async fn load_users(&mut self) -> Vec<String>{
        let users_col: Collection<Document> = self.db.collection(&"users");
        let users = [ 
            doc! {
                "username": "Matias".to_string(),
                "last_login":  Bson::Null, "date_joined": Utc::now(),
                "password": hash_password(&"matias9404".to_string()),
                "email": "matias@arrobatech.com.ar".to_string(), "old_id": 1,
                "role": "Admin", "active": true},
            doc! {
                "username": "Maxi".to_string(),
                "last_login":  Bson::Null, "date_joined": Utc::now(),
                "password": hash_password(&"maxi9404".to_string()),
                "email": "maxi@arrobatech.com.ar".to_string(), "old_id": 2,
                "role": "Admin", "active": true},
            
            ];
        let res = users_col.insert_many(users, None).await.unwrap();
        let mut str_obj_ids: Vec<String> = Vec::new();
        for item in res.inserted_ids.iter(){
            let obj_id = item.1.as_object_id().unwrap();
            let user_items = self.items.get_mut("users");
            match user_items{
                Some(col) => {
                    col.push(obj_id)
                },
                None =>{
                    let mut users: Vec<ObjectId> = Vec::new();
                    users.push(obj_id);
                    self.items.insert("users".to_string(), users);

                }
            }
            str_obj_ids.push(obj_id.to_hex());

        }
        str_obj_ids

    
    }

    pub async fn load_user(&mut self, email: &str, role: Option<String>) -> String {
        // guardar los ids de cada collection y dropearlos
        let users_col: Collection<Document> = self.db.collection(&"users");
        let admin = doc! {
            "username": "Matias".to_string(),
            "last_login":  Bson::Null, "date_joined": Utc::now(),
            "password": hash_password(&"matias9404".to_string()),
            "email": email.to_string(), "old_id": 1,
            "role": role.unwrap_or("tech".to_string()), "active": true
        };
        let res = users_col.insert_one(admin, None).await.unwrap();
        let obj_id = res.inserted_id.as_object_id().unwrap();
        let user_items = self.items.get_mut("users");
        match user_items{
            Some(col) => {
                col.push(obj_id)
            },
            None =>{
                let mut users: Vec<ObjectId> = Vec::new();
                users.push(obj_id);
                self.items.insert("users".to_string(), users);

            }
        }
        // self.items.insert("users")
        obj_id.to_hex()

    }
    pub async fn create_customer(&mut self, customer: Customer) -> String{
        let customers_col: Collection<Customer> = self.db.collection(&"customers");
        let cus = customers_col.insert_one(customer, None).await.unwrap();
        let obj_id = cus.inserted_id.as_object_id().unwrap();
        let customer_items = self.items.get_mut("customers");
        match customer_items{
            Some(col) => {
                col.push(obj_id)
            },
            None =>{
                let mut customers: Vec<ObjectId> = Vec::new();
                customers.push(obj_id);
                self.items.insert("customers".to_string(), customers);
            }
        }
        obj_id.to_hex()

    }

    pub async fn clean(&mut self){
        // clear users:
        let user_ids = self.items.get("users");
        if let Some(users) = user_ids{
            let users_col = self.db.collection::<User>("users");
            _ = users_col.delete_many(doc!{"_id": {"$in": users}}, None).await;
        };

        
        let customer_ids = self.items.get("customers");
        if let Some(customers) = customer_ids{
            let customers_col = self.db.collection::<Customer>("customers");
            let query = doc!{"_id": {"$in": customers}};
            _ = customers_col.delete_many(query, None).await;
        };
        
        let repair_ids = self.items.get("repairs");
        if let Some(repair) = repair_ids{
            let repairs_col = self.db.collection::<Repair>("repairs");
            _ = repairs_col.delete_many(doc!{"_id": {"$in": repair}}, None).await;
        };

    }
    pub async fn load_repairs(&self){
        todo!()
    }
    pub async fn load_customers(&self){
        todo!()
    }
}




pub struct CustomerBuilder{
    name: String,
    repaired_products: Vec<RepairedProduct>
}

impl CustomerBuilder{
    pub fn new() -> Self{
        Self{
            name: "test_customer".to_string(),
            repaired_products: vec![]
        }

        
    }
    pub fn with_repaired_products(mut self, repaired_products: Vec<RepairedProduct>)->Self {
        self.repaired_products = repaired_products;
        self
    }
    pub fn name(mut self, name: String) -> Self{
        self.name = name;
        self

    }
    pub fn build(&self) -> Customer{
        Customer{
            name: self.name.clone(),
            repaired_products: self.repaired_products.to_owned(),
            ..Default::default()
        }
    }
}




pub async fn create_dummy_repair(
    technician: &ObjectId,
    db: &Database,
    prod_type: String,
    status: String,
    user_id: &ObjectId,
    log_entry: Log
)->(ObjectId, ObjectId, ObjectId){
    
    let rep_id = ObjectId::new();
    let cus_id = ObjectId::new();
    let prod_id = ObjectId::new();
    let customer = doc!{
        "_id": cus_id,
        "name": cus_id.to_hex(),
        "last_name": cus_id.to_hex(),
        "location": "white bay",
        "address": "avenida siempre viva 2",
        "phone": "12345",
        "email": "si",
        "products":[doc!{
            "_id": prod_id,
            "product_type": prod_type,
            "brand": "Samsung",
            "model": "asd-123",
            "serial_number": "123-dfsdfds",
            "created_at": Utc::now()
            }
        ]
    };
    let cus_col = db.collection::<Document>("customers");
    let customer_id = match cus_col.insert_one(customer, None).await{
        Ok(res)=>{
            res.inserted_id.as_object_id().unwrap()
        },
        Err(_e)=> panic!("YOLO!")
    };
    let date =Utc::now().format("%Y-%m-%d").to_string();
    // let naive_date = NaiveDate::parse_from_str(&date, "%Y-%m-%d").unwrap();
    let mut logs: Vec<Log> = Vec::new();
    logs.push(log_entry);
    let repair_request = Repair{
        id: Some(rep_id),
        received_by:user_id.to_hex(),
        received_by_id: user_id.clone(),
        customer:customer_id,
        product:RepairedProduct{
            id: Some(prod_id),
            product_type: "cellphone".to_string(),
            brand:"Samsung".to_string(),
            model:"asd-123".to_string(),
            serial_number: Some("123-dfsdfds".to_string()),
            created_at: Utc::now()
            },
        technician: Some(technician.to_hex()),
        technician_id: Some(technician.clone()),
        repair_id:1,
        logs: logs,
        description: "no le anda".to_string(),
        status: RepairState::from(status.as_str()),
        ..Default::default()
    };
    let repairs_col = db.collection::<Repair>("repairs");
    _ = repairs_col.insert_one(repair_request, None).await.unwrap();
    (rep_id, cus_id, prod_id)
}

// crear una reparacion y devolver id de reparacion y client


pub struct LoggedClient{
    client: RClient,
    kuki: Option<Cookie<'static>>,
    pub due_cookie: bool
}

impl LoggedClient{

    pub async fn init() -> Self{
        let client = RocketClient::tracked(repair_manager::rocket().await).await.unwrap();
        Self{client, kuki: None, due_cookie: false}
    }

    pub async fn with_user(&mut self, email: &str, db: &mut DbFixture, role: Option<String>) -> String{
        let user = db.load_user(email, role).await;
        let mut creds = HashMap::new();
        creds.insert("email", email);
        creds.insert("password", "matias9404");
        let res = self.client.post("/login")
            .header(ContentType::JSON)
            .json(&creds)
            .dispatch()
            .await;
        let kuki = res.cookies().get_private("user").unwrap();
        self.kuki = Some(kuki);
        assert_eq!(res.status(), Status::Ok);
        user
    }

    pub async fn post <'a, T>(&'a self, data: &'a T, uri: String) -> LocalResponse
    where T:  Serialize
    {   
        //
        //`self` escapes the associated function body here
        //           argument requires that `'1` must outlive `'static`
        let cookie = match self.kuki.as_ref(){
            Some(c) =>c.to_owned(),
            None =>  {
                let mut cookie = Cookie::new("user", "invalid");
                let mut now = OffsetDateTime::now_utc();
                if self.due_cookie{
                    now -= Duration::hours(11);
                }else{
                    now += Duration::hours(10);
                }
                cookie.set_expires(now);
                cookie
            }
        };
        self.client.post(uri)
            .header(ContentType::JSON)
            .private_cookie(cookie)
            .json(&data)
            .dispatch()
            .await
    
    }
    pub async fn put <'a, T>(&'a self, data: &'a T, uri: String) -> LocalResponse
    where T:  Serialize
    {   
        let cookie = match self.kuki.as_ref(){
            Some(c) =>c.to_owned(),
            None =>  {
                let mut cookie = Cookie::new("user", "invalid");
                let mut now = OffsetDateTime::now_utc();
                now += Duration::hours(10);
                cookie.set_expires(now);
                cookie
            }
        };
        self.client.put(uri)
            .header(ContentType::JSON)
            .private_cookie(cookie)
            .json(&data)
            .dispatch()
            .await
    
        }
    pub async fn get(&self, uri: String) -> LocalResponse{   
        let cookie = match self.kuki.as_ref(){
            Some(c) =>c.to_owned(),
            None =>  {
                let mut cookie = Cookie::new("user", "invalid");
                let mut now = OffsetDateTime::now_utc();
                now += Duration::hours(10);
                cookie.set_expires(now);
                cookie
            }
        };
        self.client.get(uri)
            .header(ContentType::JSON)
            .private_cookie(cookie)
            .dispatch()
            .await
    
        }
}