use hyper;
use debug;
use sha2::Sha256;
use hmac::{Hmac, Mac};
use ws::connect;
use serde_json;
use reqwest;
use std;
use types::{DataRegistry, TextRegistry, DictRegistry, OrderbookSide, BidaskRegistry, BidaskReadOnlyRegistry, BidaskTextRegistry};

use std::collections::HashMap;
use dictionary::Dictionary;
use dictionary;
use Brokers::{BROKER, getKey, TASK, BROKERS};
use base64::{encode, decode};
use std::time::Instant;

pub struct RegistryData {
    pub bid: Option<String>,
    pub ask: Option<String>,
    pub last: Option<String>,
    orderbook: Universal_Orderbook,
}

impl RegistryData {
    pub fn new(bid: Option<String>, ask: Option<String>, last: Option<String>, orderbook: Universal_Orderbook) -> Self {
        RegistryData { bid: bid, ask: ask, last: last, orderbook: orderbook }
    }
    pub fn get_bids_mut(&mut self) -> &mut OrderbookSide {
        &mut self.orderbook.bids
    }
    pub fn get_bids(&self) -> &OrderbookSide {        &self.orderbook.bids    }
    pub fn has_bids(&self) -> bool {        self.orderbook.bids.len() >0    }
    pub fn has_asks(&self) -> bool {        self.orderbook.asks.len() >0    }
    pub fn get_asks_mut(&mut self) -> &mut OrderbookSide {
        &mut self.orderbook.asks
    }
    pub fn get_asks(&self) -> &OrderbookSide {
        &self.orderbook.asks
    }
    pub fn set_bids(&mut self, bids: OrderbookSide) {
        self.orderbook.bids = bids;
    }
    pub fn set_asks(&mut self, asks: OrderbookSide) {
        self.orderbook.asks = asks;
    }
    pub fn print(&self) {
        println!("{:?}", self.orderbook.bids)
    }
}

pub struct Data {
    pub bid: Option<String>,
    pub ask: Option<String>,
    pub last: Option<String>,
}

pub struct Universal_Orderbook {
    pub bids: OrderbookSide,
    pub asks: OrderbookSide,
}

impl Universal_Orderbook {
    pub fn get_bids_mut(&mut self) -> &OrderbookSide {
        &self.bids
    }
    pub fn get_bids(&self) -> &OrderbookSide {
        &self.bids
    }
    pub fn get_asks_mut(&mut self) -> &OrderbookSide {
        &self.asks
    }
    pub fn get_asks(&self) -> &OrderbookSide {
        &self.asks
    }
    pub fn print(&self) {
        println!("{:?}", self.bids)
    }
}

impl std::fmt::Debug for Universal_Orderbook {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut st = "";
        try!(fmt.write_str("bids:["));
        for i in self.bids.iter() {
            try!(fmt.write_str(st));
            try!(fmt.write_str(&format!("{:?}", i)));
            st = ",";
        }
        let mut st = "";
        try!(fmt.write_str("],asks:["));
        for i in self.bids.iter() {
            try!(fmt.write_str(st));
            try!(fmt.write_str(&format!("{:?}", i)));
            st = ",";
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

fn parse_response_depth(task: TASK, broker: BROKER, text: String) -> Universal_Orderbook {
    let mut r: Universal_Orderbook;
    match task {
        TASK::HTTP_DEPTH => {
            match broker {
                BROKER::BINANCE => {//todo
                    let text2 = str::replace(&text, ",[]", "");
                    r = Universal_Orderbook { bids: HashMap::new(), asks: HashMap::new() };
                    //r = text2;
                }
                BROKER::KUCOIN => {
                    r = kucoin::parse_depth(text);
                }
                _ => {
                    r = Universal_Orderbook { bids: HashMap::new(), asks: HashMap::new() };
                }
            }
        }
        _ => {
            r = Universal_Orderbook { bids: HashMap::new(), asks: HashMap::new() };
        }
    }
    r
}

pub fn fetch_bidask(broker: BROKER) -> HashMap<String, Data> {
    let url = get_url(TASK::HTTP_BIDASK, broker, "".to_string());
    debug::print_fetch(broker, &url);
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

pub fn fetch_depth(broker: BROKER, infra: &String, supra: &String, DICT: &DictRegistry) -> Universal_Orderbook {
    let mut result: Universal_Orderbook = Universal_Orderbook { bids: HashMap::new(), asks: HashMap::new() };
    let rawnameopt = dictionary::read_rawaltname(broker, supra.to_string(), infra.to_string(), DICT);
    if rawnameopt.is_some() {
        let rawpair = rawnameopt.unwrap();
        let url = format!("{}", get_url(TASK::HTTP_DEPTH, broker, rawpair.to_string()));
        debug::print_fetch(broker, &url);
        let h = get_headers(TASK::HTTP_DEPTH, broker, infra.to_string(), supra.to_string(), "".to_string());

        let client = reqwest::Client::new();
        match client.get(&url).headers(h).send() {
            Ok(mut res) => {
                match res.text() {
                    Ok(val) => {

                        parse_response_depth(TASK::HTTP_DEPTH, broker, val)
                    }
                    Err(err) => {
                        println!("[GET_DEPTH] err");
                        result
                    }
                }
            }
            Err(err) => {
                println!("[GET_DEPTH] {:?}", err);
                result
            }
        }

    } else {
        result
    }
}

pub fn fetch_price(broker: BROKER) -> HashMap<String, Data> {
    let url = get_url(TASK::HTTP_PRICE, broker, "".to_string());
    debug::print_fetch(broker, &url);
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
    println!("listen url ws_tick {} {}", broker, url);
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
        BROKER::BITFINEX => {
            match connect(url.to_string(), |out| bitfinex::WSDepthClient { out: out, broker: broker, symbol: symbol.to_string(), registry: registry.clone() }) {
                Ok(c) => { println!("connected"); }
                Err(err) => { println!("WS Cannot connect {} {}", broker, url) }
            }
        }
        _ => { println!("err unknown broker"); }
    }
}

fn get_url(task: TASK, broker: BROKER, pair: String) -> String {
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
                BROKER::BINANCE => { r = binance::URL_HTTP_DEPTH.to_string(); }
                BROKER::KUCOIN => {
                    r = kucoin::URL_HTTP_DEPTH.to_string();
                    r = str::replace(&r, "XXX", &pair);
                }
                BROKER::KRAKEN => {
                    r = kraken::URL_HTTP_DEPTH.to_string();
                    r = str::replace(&r, "XXX", &pair);
                }
                _ => {}
            }
        }
        TASK::WS_TICK => {
            match broker {
                BROKER::BINANCE => {
                    r = binance::URL_WS_TICK.to_string();
                    r = str::replace(&r, "XXX", &pair);
                }
                _ => {}
            }
        }
        TASK::WS_DEPTH => {
            match broker {
                BROKER::BINANCE => {
                    r = binance::URL_WS_DEPTH.to_string();
                    r = str::replace(&r, "XXX", &pair);
                    r=r.to_lowercase();
                }
                BROKER::HITBTC => {
                    r = hitbtc::URL_WS_DEPTH.to_string();
                }
                BROKER::BITFINEX => {
                    r = bitfinex::URL_WS_DEPTH.to_string();
                }
                _ => {}
            }
        }
        _ => {}
    }
    r
}

fn get_headers(task: TASK, broker: BROKER, infra: String, supra: String, queryString: String) -> hyper::header::Headers {
    let mut h: hyper::header::Headers = hyper::header::Headers::new();
    ;
    match task {
        TASK::HTTP_BIDASK => {
            match broker {
                _ => {}
            }
        }
        TASK::HTTP_PRICE => {
            match broker {
                _ => {}
            }
        }
        TASK::HTTP_DEPTH => {
            match broker {
                BROKER::KUCOIN => {
                    let KEY = "5a7ac4eadf542b26bf82ef77";
                    let SECRET = "e501c6f6-ffcc-491b-852d-ff0f9638aeac";
                    let endpoint = "/open/orders";
                    let nonce = get_timestamp();
                    let strForSign = format!("{}/{}/{}", endpoint, nonce, queryString);


                    //init
                    let mut sha256_HMAC = Hmac::<Sha256>::new(SECRET.as_bytes()).unwrap();

                    //run
                    let signatureStr = encode(strForSign.as_bytes());
                    sha256_HMAC.input(signatureStr.as_bytes());
                    let result = sha256_HMAC.result();

                    let signature = to_hex_string(result.code().to_vec());


                    h.set_raw("KC-API-KEY", KEY);
                    h.set_raw("KC-API-NONCE", nonce.to_string());
                    h.set_raw("KC-API-SIGNATURE", signature);
                }
                _ => {}
            }
        }
        TASK::WS_TICK => {
            match broker {
                BROKER::BINANCE => {}
                _ => {}
            }
        }
        TASK::WS_DEPTH => {
            match broker {
                BROKER::BINANCE => {}
                BROKER::HITBTC => {}
                _ => {}
            }
        }
        _ => {}
    }
    h
}

fn get_timestamp() -> u64 {
    let start = Instant::now();
    let elapsed = start.elapsed();
    (elapsed.as_secs() * 1_000) + (elapsed.subsec_nanos() / 1_000_000) as u64
}

pub fn to_hex_string(bytes: Vec<u8>) -> String {
    let strs: Vec<String> = bytes.iter()
        .map(|b| format!("{:02X}", b))
        .collect();
    strs.connect(" ")
}