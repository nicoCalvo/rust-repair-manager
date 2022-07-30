#![allow(unused_imports, dead_code)]
use chrono::{Utc, NaiveDate};




fn main(){
    
    let lala = Utc::now().date();
    let asd =Utc::now().format("%Y-%m-%d").to_string();
    let _asd = NaiveDate::parse_from_str(&asd, "%Y-%m-%d");
    println!("{}", lala);

}