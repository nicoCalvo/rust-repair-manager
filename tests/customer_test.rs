#[allow(unused_imports)]
use rocket::{Rocket, State, Build};
use rocket::fairing::AdHoc;


mod common;

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use bson::{Document, doc};
    use mongodb::Collection;
    use rocket::{tokio, http::{ContentType, Cookie}, time::{OffsetDateTime, Duration}};
    use ::rocket::{http::Status, async_test};
    use super::Rocket;
    use ::rocket::local::asynchronous::Client;
    use crate::common::{DbFixture};


    #[async_test]
    async fn test_create_customer() {
        let cookie = Cookie::new("user_id", "62c19eaa2f50b44b9529d927");
        let mut map: HashMap<String, String> = HashMap::new();
        map.insert("name".to_string(), "pepe".to_string());

        let client = Client::tracked(repair_manager::rocket().await).await.unwrap();
        let resp = client.post("/customers")
            .header(ContentType::JSON)
            .private_cookie(cookie)
            .json(&map)
            .dispatch()
            .await;
        dbg!(resp);
        
    }
}