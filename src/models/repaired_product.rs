use serde::{Deserialize, Serialize};
use bson::{doc, Document};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RepairedProduct{
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<bson::oid::ObjectId>,
    pub _type: String,
    pub brand: String,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub serial_number: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>
    
}

