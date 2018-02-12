use Data;
use std::collections::HashMap;
use serde_json;
use Universal::{Universal_Orderbook,RegistryData};
use types::{DataRegistry, TextRegistry, DictRegistry,OrderbookSide,BidaskRegistry, BidaskReadOnlyRegistry, BidaskTextRegistry};

static NAME: &str = "kucoin";
pub static URL_HTTP_BIDASK: &str = "https://api.kucoin.com/v1/open/tick";
pub static URL_HTTP_DEPTH: &str = "https://api.kucoin.com/v1/open/orders?symbol=XXX&limit=100";

#[derive(Serialize, Deserialize)]
pub struct Bidask {
    pub success: bool,
    pub code: String,
    pub msg: String,
    pub timestamp: u64,
    pub data: Vec<kucoin_bidask_in>,
}

#[derive(Serialize, Deserialize)]
pub struct Depth {
    pub success: bool,
    pub code: String,
    pub msg: String,
    pub timestamp: u64,
    pub data: Depth_in,
}

#[derive(Serialize, Deserialize)]
pub struct Depth_in {
    SELL: Vec<Vec<f64>>,
    BUY: Vec<Vec<f64>>,
}

#[derive(Serialize, Deserialize)]
pub struct kucoin_bidask_in {
    pub coinType: String,
    pub trading: bool,
    pub symbol: String,
    pub lastDealPrice: Option<f64>,
    pub buy: Option<f64>,
    pub sell: Option<f64>,
    pub change: Option<f64>,
    pub coinTypePair: Option<String>,
    pub sort: Option<u64>,
    pub feeRate: Option<f64>,
    pub volValue: Option<f64>,
    pub high: Option<f64>,
    pub datetime: Option<u64>,
    pub vol: Option<f64>,
    pub low: Option<f64>,
    pub changeRate: Option<f64>,
}


pub fn parse_bidask(text: String) -> HashMap<String, Data> {
    let mut r = HashMap::new();
    let bs: Result<Bidask, serde_json::Error> = serde_json::from_str(&text);
    match bs {
        Ok(bs_) => {
            for row in bs_.data {
                let symb = str::replace(&row.symbol, "-", "");
                let mut b;
                if let Some(bb) = row.buy { b = Some(bb.to_string()) } else { b = None }
                let mut la;
                if let Some(la_) = row.buy { la = Some(la_.to_string()) } else { la = None }
                let mut se;
                if let Some(se_) = row.sell { se = Some(se_.to_string()) } else { se = None }
                r.insert(symb, Data { bid: b, ask: se, last: la });
            }
        }
        Err(err) => {
            println!(" !!! cannot unmarshall bidask {} {:?}", NAME, err)
        }
    }
    r
}


pub fn parse_depth(text: String) -> Universal_Orderbook {
    let parsed: Result<Depth, serde_json::Error> = serde_json::from_str(&text);
    let mut bids: OrderbookSide = HashMap::new();
    let mut asks: OrderbookSide = HashMap::new();
    match parsed {
        Ok(parsed_) => {
            for i in parsed_.data.BUY.iter() {
                    bids.insert(i[0].to_string(), i[1]);
            }
            for i in parsed_.data.SELL.iter() {
                asks.insert(i[0].to_string(), i[1]);
            }
        }
        Err(err) => {
            println!(" !!! cannot unmarshall depth {} {:?}", NAME, err)
        }
    }
    Universal_Orderbook { bids: bids, asks: asks }
}