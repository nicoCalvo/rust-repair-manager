#![allow(dead_code, unused_imports)]

use serde::{Deserialize, Serialize};
use bson::{doc, Document};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RepairedProduct{
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<bson::oid::ObjectId>,
    pub product_type: String,
    pub brand: String,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub serial_number: Option<String>
    
}

impl Default for RepairedProduct{
    fn default() -> Self {
        Self { id: Default::default(), product_type: Default::default(), brand: Default::default(), model: Default::default(), serial_number: Default::default() }
    }
}