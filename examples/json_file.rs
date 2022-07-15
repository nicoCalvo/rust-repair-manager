#![allow(dead_code)]
#[allow(unused_imports)]
use std::{env, collections::HashMap};
// How to read from json file into
// a struct splited
//
use std::path::Path;

use futures::StreamExt;
use mongodb::Collection;
use serde::{Deserialize, Serialize};

use dotenv::dotenv;


use bson::Document;

#[macro_use]
use repair_manager;
use rocket::tokio::{fs, self};

use mongodb::{
    Client, Database, bson::doc,
};



const FIXTURE_PATH: &str = "tests/fixtures";


async fn connect() -> Database{
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Query{
    collection: String,
    query: Document
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DbFixture{
    test_name: Option<String>,
    setup: Vec<Query>,
    teardown: Vec<Query>,

}

impl Drop for DbFixture{
    fn drop(&mut self) {
        println!("Im Dropping!");
    }
}


#[tokio::main]
async fn main(){
    let db = connect().await;
    let path = env::current_dir().unwrap();
    let more_path = path.as_os_str().to_string_lossy().to_string();
    let fixtures_path = Path::new(&more_path);
    let fixture_file_path = fixtures_path.join(Path::new(FIXTURE_PATH));
    let fixture = fixture_file_path.join(Path::new("login.json"));
    let asd = fixture.as_os_str().to_str().unwrap();
    println!("{}", asd.clone());
    let contents = fs::read_to_string(asd).await.unwrap();
    // let json: serde_json::Value = serde_json::from_str(&contents).expect("JSON was not well-formatted");
    let fixture: DbFixture = serde_json::from_str(&contents).expect("JSON was not well-formatted");
    // let setup_method = json.get("my_test").unwrap().get("setup").unwrap();
    
    // let collection: String = setup_method[0].get("collection").unwrap().to_string();
    let query = &fixture.setup[0];
    let col: Collection<Document> = db.collection(&query.collection);
    let cursor = col.aggregate([query.query.clone()], None).await.unwrap();
    let res = cursor.for_each(|c| async move{
        println!("{:?}", c)
    });
    res.await;
}