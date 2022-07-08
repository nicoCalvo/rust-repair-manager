#[allow(unused_imports)]
#[allow(dead_code)]
use std::env;
use std::path::Path;
use dotenv::dotenv;
use chrono::prelude::*;
use bson::{Document, Bson};
use mongodb::{Database, Collection, Client, bson::doc};
#[macro_use]
use repair_manager::models::user::User;
use repair_manager::utils::hash_password;
use rocket::tokio::{fs::File, io::AsyncReadExt};
use rocket::tokio::fs;
use serde::{Deserialize, Serialize};

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
    db: Database,

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
            db: db
        }
     
    }

    pub async fn load_users(&self){
        let users_col: Collection<Document> = self.db.collection(&"users");
        _ = users_col.drop(None).await;
     
        let users = [ doc! {
            "username": "Matias".to_string(),
            "last_login":  Bson::Null, "date_joined": Utc::now(),
            "password": hash_password(&"matias9404".to_string()),
            "email": "matias@arrobatech.com.ar".to_string(), "old_id": 1},
            doc! {
                "username": "Maxi".to_string(),
                "last_login":  Bson::Null, "date_joined": Utc::now(),
                "password": hash_password(&"maxi9404".to_string()),
                "email": "maxi@arrobatech.com.ar".to_string(), "old_id": 1},
            
            ];
        _ = users_col.insert_many(users, None).await.unwrap();
    }

    pub async fn load_repairs(&self){
        todo!()
    }
    pub async fn load_customers(&self){
        todo!()
    }
}

impl Drop for DbFixture{
    fn drop(&mut self){}
}

