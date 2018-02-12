use Brokers::BROKER;
use colored::*;

pub fn print_ws_message(broker:BROKER,symbol:&String,msg:&String){
//    println!("{}",format!("{}>{}  ->{}",broker,symbol,msg).cyan());
}
pub fn print_open_ws(broker:BROKER,symbol:&String,msg:&String){
    //println!("{}",format!("{}>{}  open ws {}",broker.to_string(),symbol,msg).cyan());
}
pub fn print_write_depth(broker:BROKER,symbol:&String,msg:&String){

    //println!("{}",format!("{}>{}  write depth {}",broker,symbol,msg).red());
}
pub fn print_read_depth(broker:BROKER,symbol:&String,msg:&String){
    //println!("{}",format!("{}>{}  read depth {}",broker.to_string(),symbol,msg).magenta());
}
pub fn print_fetch(broker:BROKER,url:&String){
  //  println!("{}",format!("{} fetch {}",broker.to_string(),url).yellow());
}
