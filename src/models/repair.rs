#![allow(dead_code, unused_imports)]
use bson::Document;
use bson::oid::ObjectId;
use ::serde::{Serialize, Deserialize};


use super::repaired_product::RepairedProduct;
use chrono::{NaiveDate, serde, Utc};

mod date_format{
    use chrono::NaiveDate;
    use serde::{Deserialize, Serialize, Serializer, Deserializer};
    const FORMAT: &'static str = "%Y-%m-%d";
    pub fn serialize<S>(date: &NaiveDate,serializer: S) -> Result<S::Ok, S::Error>
    where
    S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D,) -> Result<NaiveDate, D::Error>
    where
    D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NaiveDate::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
    }
}




#[derive(Serialize, Deserialize, Debug)]
pub struct Log{
    pub entry: String,
    #[serde(with = "repair_state")]
    pub status: RepairState,
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
    pub received_by: String,
    pub received_by_id: ObjectId,
    pub customer: bson::oid::ObjectId,
    pub product: RepairedProduct,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub technician: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub technician_id: Option<ObjectId>,
    pub logs: Vec<Log>,
    #[serde(with = "repair_state")]
    pub status: RepairState,
    pub description: String,
    pub additional: String,
    pub suggested_price: i32,
    pub warranty: i32,
    pub received_date: chrono::DateTime<chrono::Utc>,
    #[serde(with = "date_format")]
    pub estimated_fixed_date: chrono::NaiveDate,//   "received_date":  bson::Bson::DateTime(DateTime::from(Utc::now())),
    //                                               "estimated_fixed_date": Utc::now().date().format("%Y-%m-%d").to_string(),
    pub finished_repair: Option<chrono::DateTime<chrono::Utc>>, 
    pub delivered_date: Option<chrono::DateTime<chrono::Utc>>,
    pub voided_date: Option<chrono::DateTime<chrono::Utc>>,
    pub bill: Option<Bill>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub billed_by_id: Option<ObjectId>,
    pub billed_by: Option<String>,
    pub voided: bool,
    pub repair_id: i32 // rename from old_id in migration project
    
}


impl Default for Repair{
    fn default() -> Self {
        let date =Utc::now().format("%Y-%m-%d").to_string();
        let naive_date = NaiveDate::parse_from_str(&date, "%Y-%m-%d").unwrap();
        Self { 
            id: None,
            received_by: "Undefined".to_string(),
            received_by_id: Default::default(),
            customer: Default::default(),
            product: Default::default(),
            technician: Default::default(),
            technician_id: Default::default(),
            logs: Default::default(),
            status: RepairState::Received,
            description: Default::default(),
            additional: Default::default(),
            suggested_price: Default::default(),
            warranty:6,
            received_date: Utc::now(),
            estimated_fixed_date:naive_date,
            finished_repair: Default::default(),
            delivered_date: Default::default(),
            voided_date: Default::default(),
            bill: Default::default(),
            billed_by_id: Default::default(),
            billed_by: Default::default(),
            voided: Default::default(),
            repair_id: Default::default() 
        }
    }
}




#[derive(PartialEq, Serialize, Deserialize, Debug, Clone, FromFormField)]
pub enum RepairState {
    Received,
    InProgress,
    Delivered,
    Voided,
    Budget,
    Derived,
    ToBeDelivered,
    NotRepaired,
    WaitingSpare
}


impl From<&str> for RepairState{
    fn from(status: &str) -> Self {
        match status{
            "Recibida" => RepairState::Received,
            "En progreso" => RepairState::InProgress,
            "Entregada" => RepairState::Delivered,
            "Anulada" => RepairState::Voided,
            "Esperando repuesto" => RepairState::WaitingSpare,
            "Confirmacion presupuesto" => RepairState::Budget,
            "Derivada" => RepairState::Derived,
            "Para entregar" => RepairState::ToBeDelivered,
            "Sin reparar" => RepairState::NotRepaired,
            _ => unreachable!()
        }
    }
}

impl Into<String> for &RepairState{
    fn into(self) -> String {
        match self{
            RepairState::Received => "Recibida".to_string(),
            RepairState::InProgress => "En progreso".to_string(),
            RepairState::Delivered => "Entregada".to_string(),
            RepairState::Voided => "Anulada".to_string(),
            RepairState::Budget => "Confirmacion presupuesto".to_string(),
            RepairState::Derived => "Derivada".to_string(),
            RepairState::ToBeDelivered => "Para entregar".to_string(),
            RepairState::NotRepaired => "Sin reparar".to_string(),
            RepairState::WaitingSpare => "Esperando repuesto".to_string()
        }
    }
}

impl Into<String> for RepairState{
    fn into(self) -> String{
        let rep_state_ref = &self;
        let res: String = rep_state_ref.into();
        res
    }
}



mod repair_state{
    use super::RepairState;
    use serde::{Deserialize, Serialize, Serializer, Deserializer};
    pub fn serialize<S>(status: &RepairState,serializer: S) -> Result<S::Ok, S::Error>
    where
    S: Serializer,
    {   
        let string_status: String = status.into();
        serializer.serialize_str(&string_status)
    }

    pub fn deserialize<'de, D>(deserializer: D,) -> Result<RepairState, D::Error>
    where
    D: Deserializer<'de>,
    {
        
        let s = String::deserialize(deserializer)?;
        Ok(RepairState::from(s.as_str()))
    }
}