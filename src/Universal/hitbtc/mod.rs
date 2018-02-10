use Data;
use std::collections::HashMap;
use serde_json;

static NAME:&str="hitbtc";

#[derive(Serialize, Deserialize)]
pub struct Bidask {
    ask: Option<String>,
    bid: Option<String>,
    last: Option<String>,
    open: Option<String>,
    low: Option<String>,
    high: Option<String>,
    volume: Option<String>,
    volumeQuote: Option<String>,
    timestamp: String,
    symbol: String,
}


pub fn get_bidask(text:String) -> HashMap<String,Data>{
    let mut r = HashMap::new();

    let bs: Result<Vec<Bidask>, serde_json::Error> = serde_json::from_str(&text);
    match bs {
        Ok(bs_) =>{
            for row in bs_ {
                r.insert(row.symbol, Data { bid: row.bid, ask: row.ask, last: row.last });
            }
        },
        Err(err) => {
            println!(" !!! cannot unmarshall bidask {} {:?}", NAME, err)
        }
    }
    r
}