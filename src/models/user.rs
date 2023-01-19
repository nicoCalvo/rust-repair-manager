use bson::Document;
use rocket::serde::Deserialize;
use rocket::serde::Serialize;
use mongodb::bson::DateTime;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum Role{
    Admin,
    Technician,
    Sales
}

impl From<&str> for Role{
    fn from(role: &str) -> Self {
        match role{
            "Admin" => Role::Admin,
            "Technician" => Role::Technician,
            "Sales" => Role::Sales,
            _ => unreachable!()
        }
    }
}


impl Into<String> for Role{
    fn into(self) -> String {
        match self{
            Role::Admin => "Admin".to_string(),
            Role::Technician => "Technician".to_string(),
            Role::Sales => "Sales".to_string(),
        }
    }
}
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
    pub old_id: Option<i32>,
    pub role: Role,
    pub active: bool
}


impl From<Document> for User{
    fn from(doc: Document) -> Self {
        User{
            id: None,
            username: doc.get("username").unwrap().to_string(),
            email: Some(doc.get_str("email").unwrap_or("").to_string()),
            role: Role::from(doc.get_str("role").unwrap_or("Technician")),
            date_joined: bson::DateTime::now(),
            password: doc.get_str("password").unwrap().to_string(),
            last_login: None,
            old_id: None,
            active: true
            }
    }
}

#[cfg(test)]
mod test{
    use bson::{doc, Bson};
    use chrono::Utc;

    use super::*;

    #[test]
    fn test_serialize(){
        let user_doc = doc! {
            "username": "pepe".to_string(), "last_login": Bson::Null, "date_joined": Utc::now(),
            "password": "some_pass".to_string(), "email": "mail@mail.com".to_string(),
            "old_id": 1, "active": true, "role": "Admin"};
        let user: User = bson::from_bson::<User>(bson::Bson::Document(user_doc)).unwrap();
        assert_eq!(user.username, "pepe".to_string());
        assert_eq!(user.last_login, None);
    }
}