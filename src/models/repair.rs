#[allow(unused_imports, dead_code)]
use serde::{Deserialize, Serialize};


use super::repaired_product::RepairedProduct;

#[derive(Serialize, Deserialize, Debug)]
pub struct Log{
    pub entry: String,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub by: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Bill{
    pub amount: i32,
    pub pay_method: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub by: bson::oid::ObjectId,

}

#[derive(Serialize, Deserialize, Debug)]
pub struct Repair {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<bson::oid::ObjectId>,
    pub customer: bson::oid::ObjectId,
    pub product: RepairedProduct,
    #[serde(rename= "technician", skip_serializing_if = "Option::is_none")]
    pub technician: Option<String>,
    pub logs: Vec<Log>,
    pub status: String,
    pub  description: String,
    pub additional: String,
    pub suggested_price: i32,
    pub warranty: i16,
    pub received_date: chrono::DateTime<chrono::Utc>,
    pub estimated_fixed_date: chrono::NaiveDate,
    pub finished_repair: Option<chrono::DateTime<chrono::Utc>>,
    pub delivered_date: Option<chrono::DateTime<chrono::Utc>>,
    pub voided_date: Option<chrono::DateTime<chrono::Utc>>,
    pub bill: Option<Bill>,
    pub voided: bool,
    pub old_id: i32,
    pub received_by: String
    
}