#![allow(dead_code, unused_imports)]
use bson::{oid::ObjectId, doc, Bson};
use serde::{Serialize, Deserialize};

use repair_manager::models::customer::Customer;




#[derive(Serialize, Deserialize, Debug, Clone)]
struct Algo{
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<bson::oid::ObjectId>,
    name: String,
    last: String
}



fn main(){
    // convert string to ObjectId
    // let obj_str = "62cf748ccd15cc42b3dae315";
    // _ = dbg!(ObjectId::parse_str(obj_str));

    

    // convert doc to struct
    let _my_doc = doc!{
        "name": "yo",
        "last_name": "mismo",
        "location": "none",
        "street": "customer_data.street.clone()",
        "number": "customer_data.number.clone()",
        "phone": "customer_data.phone.clone()",
    };
    // my_doc.extend_one(doc!{"repaired_products": Vec::new()});  => experimental API 
    // let asd = Bson::Document(my_doc);
    // let cus: Customer = bson::from_bson::<Customer>(asd).unwrap();
    // dbg!(cus);


    // convert struct to doc (exclude id)
    let algo = Algo{id: Some(ObjectId::new()), name: "asdas".to_string(), last: "sada".to_string()};
    let other = Algo{id: None, ..algo};
    dbg!(other);

}