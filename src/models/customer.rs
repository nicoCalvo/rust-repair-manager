
use serde::{Deserialize, Serialize};

use super::repaired_product::RepairedProduct;


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Customer {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<bson::oid::ObjectId>,
    pub old_id: Option<i32>,
    pub name: String,
    pub last_name: String,
    pub location: String,
    pub  street: String,
    pub number: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    pub repaired_products: Vec<RepairedProduct>,
}


impl Default for Customer{
    fn default() -> Self {
        Self {
            id: None,
            old_id: None,
            name: "No declarado".to_string(),
            last_name: "No declarado".to_string(),
            location: "Bahia Blanca".to_string(),
            street: "No declarada".to_string(),
            number: "".to_string(),
            phone: None,
            email: None,
            repaired_products: Vec::new()
        }
    }
}


#[cfg(test)]
mod test{
    use super ::*;

    #[test]
    fn test_esta(){
        let customer = Customer{name: "pepe".to_string(), ..Default::default()};
        assert_eq!(customer.location, "Bahia Blanca".to_string());
        assert_eq!(customer.last_name, "No declarado".to_string());
        assert_eq!(customer.name, "pepe".to_string())
    }
    

}