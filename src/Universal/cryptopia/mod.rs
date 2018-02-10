use Data;
use std::collections::HashMap;
use serde_json;

static NAME:&str="cryptopia";

#[derive(Serialize, Deserialize)]
pub struct Bidask {
    Success: bool,
    Message: Option<String>,
    Error: Option<String>,
    Data: Vec<cryptopia_bidask_in>,
}

#[derive(Serialize, Deserialize)]
pub struct cryptopia_bidask_in {
    TradePairId: u64,
    Label: String,
    AskPrice: f64,
    BidPrice: f64,
    Low: f64,
    High: f64,
    Volume: f64,
    LastPrice: f64,
    BuyVolume: f64,
    SellVolume: f64,
    Change: f64,
    Open: f64,
    Close: f64,
    BaseVolume: f64,
    BuyBaseVolume: f64,
    SellBaseVolume: f64,
}


pub fn get_bidask(text:String) -> HashMap<String,Data>{
    let mut r = HashMap::new();
    let bs: Result<Bidask, serde_json::Error> = serde_json::from_str(&text);
    match bs {
        Ok(bs_) => {
            for row in bs_.Data {
                let symb = str::replace(&row.Label, "/", "");
                r.insert(symb, Data { bid: Some(row.BidPrice.to_string()), ask: Some(row.AskPrice.to_string()), last: Some(row.LastPrice.to_string()) });
            }
        }
        Err(err) => {
            println!(" !!! cannot unmarshall bidask {} {:?}", NAME,err)
        }
    }
    r
}