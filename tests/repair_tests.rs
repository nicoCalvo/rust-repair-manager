use rocket::{Rocket, State, Build};
use rocket::fairing::AdHoc;


mod common;

// #[cfg(test)]
// mod test {
//     use std::collections::HashMap;

//     use bson::{Document, doc};
//     use mongodb::Collection;
//     use rocket::{tokio, http::ContentType};
//     use ::rocket::{http::Status, async_test};
//     use super::Rocket;
//     use ::rocket::local::asynchronous::Client;
//     use crate::common::{DbFixture};


//     #[async_test]
//     async fn test_create_repair() {
//         let db = DbFixture::new().await;
//         db.load_users().await;
//         let client = Client::tracked(repair_manager::rocket().await).await.unwrap();
//         let mut map = HashMap::new();
//         map.insert("email", "matias@arrobatech.com.ar");
//         map.insert("password", "matias9404");
//         let resp = client.post("/login")
//             .header(ContentType::JSON)
//             .json(&map)
//             .dispatch()
//             .await;
//         let cookies = resp.cookies();


//     }
// }