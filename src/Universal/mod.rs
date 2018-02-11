use ws::connect;
use serde_json;
use reqwest;
use std;
use RefreshData;
use std::collections::HashMap;
use Brokers::{BROKER, getKey, TASK};
use DataRegistry;
use OrderbookSide;

pub struct RegistryData {
    pub bid: Option<String>,
    pub ask: Option<String>,
    pub last: Option<String>,
    orderbook:Universal_Orderbook
}
impl RegistryData{
    pub fn new(bid:Option<String>,ask:Option<String>,last:Option<String>,orderbook:Universal_Orderbook) -> Self{
        RegistryData {bid:bid,ask:ask,last:last,orderbook:orderbook}
    }
    pub fn get_bids_mut(&mut self) -> &mut OrderbookSide{
        &mut self.orderbook.bids
    }
    pub fn get_bids(&self) -> &OrderbookSide{
        &self.orderbook.bids
    }
    pub fn get_asks_mut(&mut self) -> &mut OrderbookSide{
        &mut self.orderbook.asks
    }
    pub fn get_asks(&self) -> &OrderbookSide{
        &self.orderbook.asks
    }
    pub fn set_bids(&mut self,bids:OrderbookSide){
        self.orderbook.bids=bids;
    }
    pub fn set_asks(&mut self,asks:OrderbookSide){
        self.orderbook.asks=asks;
    }
    pub fn print(&self){
       println!("{:?}",self.orderbook.bids)
    }
}

pub struct Data {
    pub bid: Option<String>,
    pub ask: Option<String>,
    pub last: Option<String>,
}

pub struct Universal_Orderbook {
    pub bids: OrderbookSide,
    pub asks: OrderbookSide
}
impl Universal_Orderbook{
    pub fn get_bids_mut(&mut self) -> &OrderbookSide{
        &self.bids
    }
    pub     fn get_bids(&self) -> &OrderbookSide{
        &self.bids
    }
    pub fn get_asks_mut(&mut self) -> &OrderbookSide{
        &self.asks
    }
    pub fn get_asks(&self) -> &OrderbookSide{
        &self.asks
    }
    pub fn print(&self){
        println!("{:?}",self.bids)
    }
}
impl std::fmt::Debug for Universal_Orderbook {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut st="";
        try!(fmt.write_str("bids:["));
        for i in self.bids.iter(){
            try!(fmt.write_str(st));
            try!(fmt.write_str(&format!("{:?}",i)));
            st=",";
        }
        let mut st="";
        try!(fmt.write_str("],asks:["));
        for i in self.bids.iter(){
            try!(fmt.write_str(st));
            try!(fmt.write_str(&format!("{:?}",i)));
            st=",";
        }

        try!(fmt.write_str("]"));

        Ok(())
    }
}

#[derive(Clone)]
pub struct Universal_Orderbook_in {
    pub price: String,
    pub size: String,
}

impl std::fmt::Debug for Universal_Orderbook_in {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
            try!(fmt.write_str("["));
            try!(fmt.write_str(&self.price));
            try!(fmt.write_str(","));
            try!(fmt.write_str(&self.size));
            try!(fmt.write_str("]"));
        Ok(())
    }
}

mod binance;
mod bitfinex;
mod hitbtc;
mod cryptopia;
mod kucoin;
mod kraken;

type RawHTTPResponse = String;

fn parse_response(task: TASK, broker: BROKER, text: RawHTTPResponse) -> HashMap<String, Data> {
    match task {
        TASK::HTTP_BIDASK => {
            match broker {
                BROKER::BITFINEX => { bitfinex::parse_bidask(text) }
                BROKER::HITBTC => { hitbtc::parse_bidask(text) }
                BROKER::CRYPTOPIA => { cryptopia::parse_bidask(text) }
                BROKER::KRAKEN => { kraken::parse_bidask(text) }
                BROKER::KUCOIN => { kucoin::parse_bidask(text) }
                BROKER::BINANCE => { binance::parse_bidask(text) }
                _ => { HashMap::new() }
            }
        }
        TASK::HTTP_PRICE => {
            match broker {
                BROKER::BINANCE => { binance::parse_price(text) }
                _ => { HashMap::new() }
            }
        }
        _ => {
            HashMap::new()
        }
    }
}

fn parse_response_depth(task: TASK, broker: BROKER, text: String) -> String {
    let mut r: String = "".to_string();
    match task {
        TASK::HTTP_DEPTH => {
            match broker {
                BROKER::BINANCE => {
                    let text2 = str::replace(&text, ",[]", "");

                    r = text2;
                }
                _ => {}
            }
        }
        _ => {}
    }
    r
}

pub fn fetch_bidask(broker: BROKER) -> HashMap<String, Data> {
    let url = get_url(TASK::HTTP_BIDASK, broker, "".to_string());
    //println!("fetch bidask {} {}", broker,url);
    let mut result: HashMap<String, Data>;
    if let Ok(mut res) = reqwest::get(&url) {
        let getres = match res.text() {
            Ok(val) => {
                let v = parse_response(TASK::HTTP_BIDASK, broker, val);
                //          println!("fetch bidask {} : get ok", broker);
                result = v;
            }
            Err(err) => {
                println!("[parse_bidask] err");
                result = HashMap::new();
            }
        };
    } else {
        result = HashMap::new();
    }
    result
}

pub fn fetch_depth(broker: BROKER, pair: &String) -> String {
    //println!("fetch string {}", broker);
    let url = format!("{}{}", get_url(TASK::HTTP_DEPTH, broker, "".to_string()), pair);
    let mut result: String;
    if let Ok(mut res) = reqwest::get(&url) {
        let getres = match res.text() {
            Ok(val) => {
                let v = parse_response_depth(TASK::HTTP_DEPTH, broker, val);
                println!("{} {}", broker, broker);
                result = v;
            }
            Err(err) => {
                println!("[GET_DEPTH] err");
                result = "".to_string()
            }
        };
    } else {
        result = "".to_string()
    }
    result
}

pub fn fetch_price(broker: BROKER) -> HashMap<String, Data> {
    //println!("fetch price {}",broker);
    let url = get_url(TASK::HTTP_PRICE, broker, "".to_string());
    let result: HashMap<String, Data>;
    if let Ok(mut res) = reqwest::get(&url) {
        let getres = match res.text() {
            Ok(val) => {
                let v = parse_response(TASK::HTTP_PRICE, broker, val);
                result = v;
            }
            Err(err) => {
                println!("[GET_PRICE] err");
                result = HashMap::new();
            }
        };
    } else {
        result = HashMap::new();
    }
    result
}

pub fn listen_ws_tick(task: TASK, broker: BROKER) {
    let url = get_url(task, broker, "ethusdt".to_string());
    println!("listen url {} {}", broker, url);
    match broker {
        BROKER::BINANCE => {
            match connect(url.to_string(), |out| binance::WSTickClient { out: out }) {
                Ok(c) => { println!("connected"); }
                Err(err) => { println!("WS Cannot connect {} {}", broker, url) }
            }
        }
        _ => { println!("err unknown broker"); }
    }
}

pub fn listen_ws_depth(task: TASK, broker: BROKER, symbol: String, registry: &DataRegistry) {
    let url = get_url(task, broker, symbol.to_string());
    println!("listen url {} {}", broker, url);
    match broker {
        BROKER::BINANCE => {
            match connect(url.to_string(), |out| binance::WSDepthClient { out: out, broker: broker, symbol: symbol.to_string(), registry: registry.clone() }) {
                Ok(c) => { println!("connected"); }
                Err(err) => { println!("WS Cannot connect {} {}", broker, url) }
            }
        }
        BROKER::HITBTC => {
            match connect(url.to_string(), |out| hitbtc::WSDepthClient { out: out, broker: broker, symbol: symbol.to_string(), registry: registry.clone() }) {
                Ok(c) => { println!("connected"); }
                Err(err) => { println!("WS Cannot connect {} {}", broker, url) }
            }
        }
        _ => { println!("err unknown broker"); }
    }
}

fn get_url(task: TASK, broker: BROKER, symbol: String) -> String {
    let mut r = "".to_string();
    match task {
        TASK::HTTP_BIDASK => {
            match broker {
                BROKER::BINANCE => { r = binance::URL_HTTP_BIDASK.to_string(); }
                BROKER::HITBTC => { r = hitbtc::URL_HTTP_BIDASK.to_string(); }
                BROKER::KUCOIN => { r = kucoin::URL_HTTP_BIDASK.to_string(); }
                BROKER::KRAKEN => { r = kraken::URL_HTTP_BIDASK.to_string(); }
                BROKER::CRYPTOPIA => { r = cryptopia::URL_HTTP_BIDASK.to_string(); }
                BROKER::BITFINEX => { r = bitfinex::URL_HTTP_BIDASK.to_string(); }
                _ => {}
            }
        }
        TASK::HTTP_PRICE => {
            match broker {
                BROKER::BINANCE => { r = binance::URL_HTTP_PRICE.to_string(); }
                _ => {}
            }
        }
        TASK::HTTP_DEPTH => {
            match broker {
                BROKER::BINANCE => { r = binance::URL_HTTP_PRICE.to_string(); }
                _ => {}
            }
        }
        TASK::WS_TICK => {
            match broker {
                BROKER::BINANCE => {
                    r = binance::URL_WS_TICK.to_string();
                    r = str::replace(&r, "XXX", &symbol)
                }
                _ => {}
            }
        }
        TASK::WS_DEPTH => {
            match broker {
                BROKER::BINANCE => {
                    r = binance::URL_WS_DEPTH.to_string();
                    r = str::replace(&r, "XXX", &symbol)
                }
                BROKER::HITBTC => {
                    r = hitbtc::URL_WS_DEPTH.to_string();

                }
                _ => {}
            }
        }
        _ => {}
    }
    r
}
