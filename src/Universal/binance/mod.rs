use Data;
use std::collections::HashMap;
use serde_json;

static NAME:&str="binance";

#[derive(Serialize, Deserialize)]
pub struct Depth {
    pub lastUpdateId: String,
    pub bids: Vec<Vec<f64>>,
    pub asks: Vec<Vec<f64>>,
}

#[derive(Serialize, Deserialize)]
pub struct Bidask {
    pub symbol: String,
    pub bidPrice: String,
    pub bidQty: String,
    pub askPrice: String,
    pub askQty: String,
}

#[derive(Serialize, Deserialize)]
pub struct Price {
    pub symbol: String,
    pub price: String,
}


pub fn get_bidask(text: String) -> HashMap<String, Data> {
    let mut r = HashMap::new();
    let bs: Result<Vec<Bidask>, serde_json::Error> = serde_json::from_str(&text);
    match bs {
        Ok(bs_) => {
            for row in bs_ {
                r.insert(row.symbol, Data { bid: Some(row.bidPrice), ask: Some(row.askPrice), /* bidq: Some(row.bidQty), askq: Some(row.askQty),*/ last: None });
            }
        },
        Err(err) => {
            println!(" !!! cannot unmarshall bidask {} {:?}", NAME, err)
        }
    }
    r
}

pub fn get_price(text:String) -> HashMap<String,Data>{
    let mut r = HashMap::new();
    let bs: Result<Vec<Price>, serde_json::Error> = serde_json::from_str(&text);
    match bs {
        Ok(bs_) => {
            for row in bs_ {
                r.insert(row.symbol, Data { bid: None, ask: None, last: Some(row.price) });
            }
        },
        Err(err) => {
            println!(" !!! cannot unmarshall price {}{:?}", NAME, err)
        }
    }
    r
}