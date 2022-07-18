use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize, Debug, Clone)]
struct Algo{
    name: String,
    last: String
}


fn main(){

    let mut bucket: Vec<Algo> = Vec::new();
    bucket.push(Algo{name: "pepe".to_string(), last: "asd".to_string()});
    bucket.push(Algo{name: "pepe2".to_string(), last: "asd2".to_string()});
    bucket.push(Algo{name: "pepe3".to_string(), last: "asd3".to_string()});

    let alguito = bucket.iter()
        .find(|item|{
            item.name == "pepe3" && item.last == "asd3"
        });
    if let Some(alg) = alguito{
        dbg!(alg);
    }else{
        dbg!("not found");
    }
}