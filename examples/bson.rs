use bson::oid::ObjectId;




fn main(){
    let obj_str = "62cf748ccd15cc42b3dae315";
    dbg!(ObjectId::parse_str(obj_str));
}