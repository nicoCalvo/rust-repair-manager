use mongodb::{
    Client, Database, bson::doc,
};

use dotenv::dotenv;
use std::env;

pub struct DbPool {
    pub mongo: Database,
}


pub async fn connect() -> Database {
    dotenv().ok(); 
    let env = env::var("ENVIRONMENT").unwrap_or("PROD".to_string());
    let database_url = env::var("MONGO_DB_URL")
        .expect("MONGO_DB_URL must be set");
    println!("MONGO URL: {}", database_url);
  
    let client = Client::with_uri_str(&database_url).await
        .expect(&format!("Unable to connect to DB: {}", database_url));

        
    client
        .database("admin")
        .run_command(doc! {"ping": 1}, None)
        .await.expect("Unable to reach server");
    println!("Connected successfully.");
    if env == "PROD" {
        println!("PRODUCTION ENVIRONMENT!");
        client.database("arrobatech")
    }else {
        println!("TESTING ENVIRONMENT!");
        client.database("test_arrobatech")
    }
    
}