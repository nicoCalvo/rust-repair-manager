// use chrono::{DateTime, Utc};
// use serde::{Serialize, Deserialize};

// #[derive(Serialize, Deserialize, Debug)]
// pub struct StructWithCustomDate {
//     // DateTime supports Serde out of the box, but uses RFC3339 format. Provide
//     // some custom logic to make it use our desired format.
//     #[serde(with = "my_date_format")]
//     pub timestamp: DateTime<Utc>,

//     // Any other fields in the struct.
//     pub bidder: String,
// }
// mod my_date_format {
//     use chrono::{DateTime, Utc, TimeZone};
//     use serde::{self, Deserialize, Serializer, Deserializer};

//     const FORMAT: &'static str = "%Y-%m-%d %H:%M:%S";

//     // The signature of a serialize_with function must follow the pattern:
//     //
//     //    fn serialize<S>(&T, S) -> Result<S::Ok, S::Error>
//     //    where
//     //        S: Serializer
//     //
//     // although it may also be generic over the input types T.
//     pub fn serialize<S>(
//         date: &DateTime<Utc>,
//         serializer: S,
//     ) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         let s = format!("{}", date.format(FORMAT));
//         serializer.serialize_str(&s)
//     }

//     // The signature of a deserialize_with function must follow the pattern:
//     //
//     //    fn deserialize<'de, D>(D) -> Result<T, D::Error>
//     //    where
//     //        D: Deserializer<'de>
//     //
//     // although it may also be generic over the output types T.
//     pub fn deserialize<'de, D>(
//         deserializer: D,
//     ) -> Result<DateTime<Utc>, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         let s = String::deserialize(deserializer)?;
//         Utc.datetime_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
//     }
// }

// #[derive(Serialize, Deserialize, Debug)]
// pub struct Algo{
//     #[serde(serialize_with="parse_from_str")]
//     pub estimated_fixed_date: chrono::NaiveDate
// }

// fn parse_from_str<'de, D>(
//     deserializer: D,
// ) -> Result<DateTime<Utc>, D::Error>
// where
//     D: Deserializer<'de>,
// {
//     dbg!(&serialized);
//     let deserialized = serde_json::from_str(&serialized).unwrap();
// }

// fn main() {
//     let alguito = doc!{
//         "estimated_fixed_date": "2015-09-05"
//     };
//     let algo: Algo = serde_json:
//     // let json_str = r#"
//     //   {
//     //     "timestamp": "2017-02-16 21:54:30",
//     //     "bidder": "Skrillex"
//     //   }
//     // "#;

//     // let data: StructWithCustomDate = serde_json::from_str(json_str).unwrap();
//     // println!("{:#?}", data);

//     // let serialized = serde_json::to_string_pretty(&data).unwrap();
//     // println!("{}", serialized);
// }

fn main(){}