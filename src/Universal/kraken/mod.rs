use Data;
use std::collections::HashMap;
use serde_json;

static NAME:&str="kraken";

#[derive(Serialize, Deserialize)]
pub struct Bidask {
    error: Vec<String>,
    result: HashMap<String, kraken_bidask_in>,
}

#[derive(Serialize, Deserialize)]
pub struct kraken_bidask_in {
    a: Option<Vec<String>>,
    b: Option<Vec<String>>,
    c: Option<Vec<String>>,
    v: Option<Vec<String>>,
    p: Option<Vec<String>>,
    t: Option<Vec<u64>>,
    l: Option<Vec<String>>,
    h: Option<Vec<String>>,
    o: Option<String>,
}


pub fn get_bidask(text:String) -> HashMap<String,Data>{

    let mut r = HashMap::new();
    let bs: Result<Bidask, serde_json::Error> = serde_json::from_str(&text);
    match bs {
        Ok(bs_) => {
            for (symbol, row) in bs_.result.iter() {
                let mut b;
                match row.b {
                    Some(ref b_) => { b = Some(b_[0].to_string()) }
                    None => { b = Some("".to_string()) }
                }
                let mut a;
                match row.a {
                    Some(ref a_) => { a = Some(a_[0].to_string()) }
                    None => { a = Some("".to_string()) }
                }
                let mut c;
                match row.c {
                    Some(ref c_) => { c = Some(c_[0].to_string()) }
                    None => { c = Some("".to_string()) }
                }
                r.insert(symbol.to_string(), Data { bid: b, ask: a, last: c });
            }
        }
        Err(err) => {
            println!(" !!! cannot unmarshall bidask {} {:?}", NAME,err)
        }
    }
    r
}