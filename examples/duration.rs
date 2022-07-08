
use std::time::SystemTime;

fn main(){
    
    let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).expect("get millis error");
    println!("now millis: {}", now.as_millis());

}