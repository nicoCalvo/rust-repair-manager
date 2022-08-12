#![allow(unused_imports, dead_code)]
use chrono::{Utc, NaiveDate, NaiveDateTime};
use rocket::time::{OffsetDateTime, Duration, format_description};




fn main(){
    let mut now = OffsetDateTime::now_utc();
    now += Duration::hours(10);
    let format = format_description::parse(
        "[year]-[month]-[day] [hour]:[minute]:[second]",
    ).unwrap();
    let enestring = now.format(&format).unwrap();
    println!("{}", enestring);
    let asd1 =  NaiveDateTime::parse_from_str(enestring.as_str(),"%Y-%m-%d %H:%M:%S").unwrap();
    // println!("{}", asd);
    let lala = Utc::now().date();
    let date = Utc::now().format("%Y-%m-%dT%H:%M:%S");
    println!("{}", date);
    // let asd =Utc::now().naive_utc().format("%Y-%m-%d %H:%M:%S").to_string();
    let asd2 =Utc::now().naive_utc();

    dbg!(&asd1);
    dbg!(&asd2);
    if asd1 > asd2{
        println!("si");
    }
    // let _asd = NaiveDate::parse_from_str(&asd, "%Y-%m-%d").unwrap();
    // println!("{}", asd);

}