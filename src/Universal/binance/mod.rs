use std;use Data;
use std::collections::HashMap;
use OrderbookSide;
use serde_json;
use Brokers::BROKER;
use Universal::Universal_Orderbook;
use Universal::Universal_Orderbook_in;
use RefreshData;
use DataRegistry;
use ws::{listen, connect, Handshake, Handler, Sender, Result as wsResult, Message, CloseCode};

static NAME: &str = "binance";
pub static URL_HTTP_BIDASK: &str = "https://api.binance.com/api/v1/ticker/bookTicker";
pub static URL_HTTP_PRICE: &str = "https://api.binance.com/api/v3/ticker/price";
pub static URL_HTTP_DEPTH: &str = "https://api.binance.com/api/v1/depth?symbol=\"";
pub static URL_WS_TICK: &str = "wss://stream.binance.com:9443/stream?streams=XXX@kline_1m";
pub static URL_WS_TRADE: &str = "wss://stream.binance.com:9443/stream?streams=XXX@trade";
pub static URL_WS_DEPTH: &str = "wss://stream.binance.com:9443/ws/XXX@depth20";

//HTTP
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

//WS TICK
#[derive(Serialize, Deserialize)]
pub struct WSTick {
    pub stream: String,
    pub data: WSTick_in,
}

#[derive(Serialize, Deserialize)]
pub struct WSTick_in {
    e: String,
    E: u64,
    s: String,
    k: WSTick_in_in,
}

#[derive(Serialize, Deserialize)]
pub struct WSTick_in_in {
    t: u64,
    T: u64,
    s: String,
    i: String,
    f: u64,
    L: u64,
    o: String,
    c: String,
    h: String,
    l: String,
    v: String,
    n: u64,
    x: bool,
    q: String,
    V: String,
    Q: String,
    B: String,
}

//WS DEPTH
#[derive(Serialize, Deserialize)]
pub struct WSDepth {
    pub lastUpdateId: u64,
    pub bids: Vec<Vec<String>>,
    pub asks: Vec<Vec<String>>,
}

//WS TICK CLIENT
pub struct WSTickClient {
    pub out: Sender,
}

impl Handler for WSTickClient {
    fn on_open(&mut self, _: Handshake) -> wsResult<()> {
        println!("WS open {}", NAME);
        Ok(())
    }
    fn on_message(&mut self, msg: Message) -> wsResult<()> {
        let mmm = msg.to_string();
        println!("WS msg {} {}", NAME, mmm);
        let mm: Result<WSTick, serde_json::Error> = serde_json::from_str(&mmm);
        match mm {
            Ok(mm_) => {
                println!("NEW P {}", mm_.data.k.c)
            }
            Err(err) => {
                println!("cannot unmarshal {} ws tick {}", NAME, err)
            }
        }

        Ok(())
        //let message: Option<String> = Universal::get_universal_msg(self, &m);
    }
    fn on_close(&mut self, code: CloseCode, reason: &str) {
        match code {
            CloseCode::Normal => println!("The client is done with the connection."),
            CloseCode::Away => { println!("The client is leaving the site. Update room count"); }
            CloseCode::Abnormal => println!("Closing handshake failed! Unable to obtain closing status from client."),
            CloseCode::Protocol => println!("protocol"),
            CloseCode::Unsupported => println!("Unsupported"),
            CloseCode::Status => { println!("Status"); }
            CloseCode::Abnormal => println!("Abnormal"),
            CloseCode::Invalid => println!("Invalid"),
            CloseCode::Protocol => println!("protocol"),
            CloseCode::Policy => println!("Policy"),
            CloseCode::Size => println!("Size"),
            CloseCode::Extension => println!("Extension"),
            CloseCode::Protocol => println!("protocol"),
            CloseCode::Restart => println!("Restart"),
            CloseCode::Again => println!("Again"),

            _ => println!("CLOSE The client encountered an error: {}", reason),
        }
    }
}

//WS DEPTH CLIENT
pub struct WSDepthClient {
    pub out: Sender,
    pub broker: BROKER,
    pub registry: DataRegistry,
    pub symbol: String,
}

impl Handler for WSDepthClient {
    fn on_open(&mut self, _: Handshake) -> wsResult<()> {
        println!("WS open {} {}", NAME,self.symbol);
        Ok(())
    }
    fn on_message(&mut self, msg: Message) -> wsResult<()> {
        let mmm = msg.to_string();
        let mmm = str::replace(&mmm, ",[]", "");
        let mm: Result<WSDepth, serde_json::Error> = serde_json::from_str(&mmm);
        match mm {
            Ok(mm_) => {
                let bid=mm_.bids.clone();
                let ask=mm_.asks.clone();

                let mut orderbook_bid: OrderbookSide = HashMap::new();
                let mut orderbook_ask: OrderbookSide = HashMap::new();

                let mut i=0;
                for  ref item  in bid.iter(){
                    i = i + 1;
                    if i < 10 {

                    orderbook_bid.insert(item[0].clone(),item[1].clone().parse::<f64>().unwrap());
                    }
                }
                let mut i=0;
                for  ref item in ask.iter(){
                    i = i + 1;
                    if i < 2 {
                        orderbook_ask.insert(item[0].clone(),item[1].clone().parse::<f64>().unwrap());
                    }
                }
                let D = Universal_Orderbook { bids: orderbook_bid, asks: orderbook_ask };
                RefreshData::snapshot_depth(self.broker, &self.registry, self.symbol.to_string(), D);

            }  Err(err) => {                println!("cannot unmarshal {} ws depth {}", NAME, err)            }
        }

        Ok(())
        //let message: Option<String> = Universal::get_universal_msg(self, &m);
    }
    fn on_close(&mut self, code: CloseCode, reason: &str) {
        match code {
            CloseCode::Normal => println!("The client is done with the connection."),
            CloseCode::Away => { println!("The client is leaving the site. Update room count"); }
            CloseCode::Abnormal => println!("Closing handshake failed! Unable to obtain closing status from client."),
            CloseCode::Protocol => println!("protocol"),
            CloseCode::Unsupported => println!("Unsupported"),
            CloseCode::Status => { println!("Status"); }
            CloseCode::Abnormal => println!("Abnormal"),
            CloseCode::Invalid => println!("Invalid"),
            CloseCode::Protocol => println!("protocol"),
            CloseCode::Policy => println!("Policy"),
            CloseCode::Size => println!("Size"),
            CloseCode::Extension => println!("Extension"),
            CloseCode::Protocol => println!("protocol"),
            CloseCode::Restart => println!("Restart"),
            CloseCode::Again => println!("Again"),

            _ => println!("CLOSE The client encountered an error: {}", reason),
        }
    }
}

pub fn parse_bidask(text: String) -> HashMap<String, Data> {
    let mut r = HashMap::new();
    let bs: Result<Vec<Bidask>, serde_json::Error> = serde_json::from_str(&text);
    match bs {
        Ok(bs_) => {
            for row in bs_ {
                r.insert(row.symbol, Data { bid: Some(row.bidPrice), ask: Some(row.askPrice), /* bidq: Some(row.bidQty), askq: Some(row.askQty),*/ last: None });
            }
        }
        Err(err) => {
            println!(" !!! cannot unmarshall bidask {} {:?}", NAME, err)
        }
    }
    r
}

pub fn parse_price(text: String) -> HashMap<String, Data> {
    let mut r = HashMap::new();
    let bs: Result<Vec<Price>, serde_json::Error> = serde_json::from_str(&text);
    match bs {
        Ok(bs_) => {
            for row in bs_ {
                r.insert(row.symbol, Data { bid: None, ask: None, last: Some(row.price) });
            }
        }
        Err(err) => {
            println!(" !!! cannot unmarshall price {}{:?}", NAME, err)
        }
    }
    r
}