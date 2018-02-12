use Data;
use std::collections::HashMap;
use serde_json;
use Universal::{Universal_Orderbook, RegistryData};
use types::{DataRegistry, TextRegistry, DictRegistry, OrderbookSide, BidaskRegistry, BidaskReadOnlyRegistry, BidaskTextRegistry};

static NAME: &str = "kraken";
pub static URL_HTTP_BIDASK: &str = "https://api.kraken.com/0/public/Ticker?pair=BCHUSD,BCHXBT,DASHUSD,DASHXBT,EOSXBT,GNOXBT,USDTZUSD,XETCXXBT,XETCZUSD,XETHXXBT,XETHZUSD,XETHZUSD.d,XICNXXBT,XLTCXXBT,XLTCZUSD,XMLNXXBT,XREPXXBT,XXBTZCAD,XXBTZCAD.d,XXBTZUSD,XXBTZUSD.d,XXDGXXBT,XXLMXXBT,XXMRXXBT,XXMRZUSD,XXRPXXBT,XXRPZUSD,XZECXXBT,XZECZUSD";
pub static URL_HTTP_DEPTH: &str = "https://api.kraken.com/0/public/Depth?pair=XXX";

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


#[derive(Serialize, Deserialize)]
pub struct Depth {
    error: Vec<String>,
    result: HashMap<String, Depth_in>,
}

#[derive(Serialize, Deserialize)]
pub struct Depth_in {
    asks: Vec<(String, String, u64)>,
    bids: Vec<(String, String, u64)>,
}

pub fn parse_bidask(text: String) -> HashMap<String, Data> {
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
            println!(" !!! cannot unmarshall bidask {} {:?}", NAME, err)
        }
    }
    r
}


pub fn parse_depth(text: String) -> Universal_Orderbook {
    println!("parsedep");
    let parsed: Result<Depth, serde_json::Error> = serde_json::from_str(&text);
    let mut bids: OrderbookSide = HashMap::new();
    let mut asks: OrderbookSide = HashMap::new();
    match parsed {
        Ok(parsed_) => {
            for (pair, val) in parsed_.result.iter() {
                for i in val.bids.iter() {
                    bids.insert(i.0.to_string(), i.1.parse::<f64>().unwrap());
                }
                for i in val.asks.iter() {
                    asks.insert(i.0.to_string(), i.1.parse::<f64>().unwrap());
                }
            }


        }
        Err(err) => {
            println!(" !!! cannot unmarshall depth {} {:?}", NAME, err)
        }
    }
    Universal_Orderbook { bids: bids, asks: asks }
}