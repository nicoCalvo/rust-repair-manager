#![allow(dead_code)]
#![allow(unused_imports)]
use rocket::{Rocket, State, Build};


mod common;

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use bson::{Document, doc};
    use mongodb::Collection;
    use rocket::{tokio, http::ContentType, time::{OffsetDateTime, Duration}};
    use ::rocket::{http::Status, async_test};
    use super::Rocket;
    use ::rocket::local::asynchronous::Client;
    use crate::common::{DbFixture};


    #[async_test]
    async fn test_login() {
        let mut db = DbFixture::new().await;
        db.load_admin().await;
        let client = Client::tracked(repair_manager::rocket().await).await.unwrap();
        let mut map = HashMap::new();
        map.insert("email", "matias@arrobatech.com.ar");
        map.insert("password", "matias9404");
        let resp = client.post("/login")
            .header(ContentType::JSON)
            .json(&map)
            .dispatch()
            .await;
        let cookies = resp.cookies();
        let user_cookie = cookies.get_private("user");
        assert!(user_cookie.is_some());
        let user_cookie = user_cookie.unwrap();
        assert!(user_cookie.http_only().unwrap());
        assert_eq!(resp.status(), Status::Ok);

        // test existing session login
        let resp = client.post("/login")
        .header(ContentType::JSON)
        .json(&map)
        .private_cookie(user_cookie)
        .dispatch()
        .await;
        assert_eq!(resp.status(), Status::Ok);

        let cookies = resp.cookies();

        // ensure cookie with user_id is set as private
        let user_cookie = cookies.get_private("user");
        assert!(user_cookie.is_some());
        let user_cookie = user_cookie.unwrap();
        dbg!(&user_cookie);
        // ensure private cookie `user_id` expires in 10 hours
        let dt_exp = user_cookie.expires_datetime().unwrap();
        let mut now = OffsetDateTime::now_utc();
        now += Duration::hours(10);
        let exp_date = (dt_exp.year(), dt_exp.month(), dt_exp.day(), dt_exp.hour());
        let exp_exp_date = (now.year(), now.month(), now.day(), now.hour());
        assert_eq!(exp_date, exp_exp_date);
        assert!(user_cookie.http_only().unwrap());
        db.clean().await
    }
  

}
