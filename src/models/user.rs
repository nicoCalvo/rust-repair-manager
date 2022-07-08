use rocket::serde::Deserialize;
use rocket::serde::Serialize;
use mongodb::bson::DateTime;


#[derive(Serialize, Deserialize, Debug)]
pub struct User{
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<bson::oid::ObjectId>,
    pub username: String,
    //https://github.com/mongodb/bson-rust/issues/293
    #[serde( skip_serializing_if = "Option::is_none")]
    pub last_login: Option<DateTime>,
    pub date_joined: DateTime,
    pub password: String,
    pub email: Option<String>,
    pub old_id: Option<i32>
}



#[cfg(test)]
mod test{
    use bson::doc;
    use chrono::Utc;

    use super::*;

    #[test]
    fn test_serialize(){
        let user_doc = doc! {
            "username": "pepe".to_string(), "last_login": Bson::Null, "date_joined": Utc::now(),
            "password": "some_pass".to_string(), "email": "mail@mail.com".to_string(), "old_id": 1};
        println!("{:?}", &user_doc);
        // user_doc.set("last_login", None);
        let user: User = bson::from_bson::<User>(bson::Bson::Document(user_doc)).unwrap();
        assert_eq!(user.username, "pepe".to_string());
        assert_eq!(user.last_login, None);
    }
}