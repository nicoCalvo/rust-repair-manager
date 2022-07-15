#![allow(dead_code, unused_imports)]
// Example of how a collection can be "casted" to a minified version of the actual data structure
//
// In this case, User is a dataset with many field, and the collection type defines only
// the ObjectId.
//
// Minifiying the amount of data being retreived

use bson::doc;
use mongodb::{bson::oid::ObjectId, options::{FindOneOptions, FindOptions}};
use repair_manager::database::db::{connect};
use rocket::tokio;
use serde::{Serialize, Deserialize};

use std::env::{set_var, remove_var};


#[derive(Debug, Default, Serialize, Deserialize)]
struct User {
    _id: ObjectId,
}


#[tokio::main]
async fn main() ->  Result<(), Box<dyn std::error::Error>>{
    set_var("ENVIRONMENT", "PROD");
    let con = connect().await;
    let users_col = con.collection::<User>("users");
    let find_opt = FindOptions::builder().projection(doc! { "_id": 1i32 }).build();
    
    let res = users_col
        .find(None,
            find_opt
        )
        .await;
    if let Ok(mut cursor) = res {
        while cursor.advance().await?{
            let id: User = cursor.deserialize_current().unwrap();
            dbg!(id);
        }
    } else {
        println!("error")
    }
    remove_var("ENVIRONMENT");
    Ok(())
}
