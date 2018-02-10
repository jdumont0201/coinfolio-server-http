use ws::{connect};
use serde_json;
use reqwest;
use std::collections::HashMap;
use TASK;

pub struct Data {
    pub bid: Option<String>,
    pub ask: Option<String>,
    pub last: Option<String>,
}

pub struct DepthData {
    pub bids: Vec<Vec<f64>>,
    pub asks: Vec<Vec<f64>>,
}

mod binance;
mod bitfinex;
mod hitbtc;
mod cryptopia;
mod kucoin;
mod kraken;

type RawHTTPResponse = String;

fn parse_response(task: TASK, broker: String, text: RawHTTPResponse) -> HashMap<String, Data> {
    match task {
        TASK::HTTP_BIDASK => {
            if broker == "bitfinex" {
                bitfinex::parse_bidask(text)
            } else if broker == "binance" {
                binance::parse_bidask(text)
            } else if broker == "hitbtc" {
                hitbtc::parse_bidask(text)
            } else if broker == "cryptopia" {
                cryptopia::parse_bidask(text)
            } else if broker == "kucoin" {
                kucoin::parse_bidask(text)
            } else if broker == "kraken" {
                kraken::parse_bidask(text)
            } else {
                HashMap::new()
            }
        }
        TASK::HTTP_PRICE => {
            if broker == "binance" {
                binance::parse_price(text)
            } else {
                HashMap::new()
            }
        }
        _ => {
            HashMap::new()
        }
    }
}

fn parse_response_depth(task: TASK, broker: String, text: String) -> String {
    let mut r: String = "".to_string();
    match task {
        TASK::HTTP_DEPTH => {
            if broker == "binance" {
                let text2 = str::replace(&text, ",[]", "");

                r = text2;
            }
        }
        _ => {}
    }
    r
}

pub fn fetch_bidask(broker: &String) -> HashMap<String, Data> {
    let url = get_url(TASK::HTTP_BIDASK, broker, "".to_string());
    //println!("fetch bidask {} {}", broker,url);
    let mut result: HashMap<String, Data>;
    if let Ok(mut res) = reqwest::get(&url) {
        let getres = match res.text() {
            Ok(val) => {
                let v = parse_response(TASK::HTTP_BIDASK, broker.to_string(), val);
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

pub fn fetch_depth(broker: &String, pair: &String) -> String {
    //println!("fetch string {}", broker);
    let url = format!("{}{}", get_url(TASK::HTTP_DEPTH, broker, "".to_string()), pair);
    let mut result: String;
    if let Ok(mut res) = reqwest::get(&url) {
        let getres = match res.text() {
            Ok(val) => {
                let v = parse_response_depth(TASK::HTTP_DEPTH, broker.to_string(), val);
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

pub fn fetch_price(broker: &String) -> HashMap<String, Data> {
    //println!("fetch price {}",broker);
    let url = get_url(TASK::HTTP_PRICE, &broker, "".to_string());
    let mut result: HashMap<String, Data>;
    if let Ok(mut res) = reqwest::get(&url) {
        let getres = match res.text() {
            Ok(val) => {
                let v = parse_response(TASK::HTTP_PRICE, broker.to_string(), val);
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

pub fn listen_ws_tick(task: TASK, broker: String) {
    let url = get_url(task, &broker, "ethusdt".to_string());
    println!("listen url {} {}", broker, url);
    if broker == "binance" {
            match connect(url.to_string(), |out| binance::WSTickClient { out: out }) {     Ok(c) => {     println!("connected");       }      Err(err) => { println!("WS Cannot connect {} {}", broker, url) } }

    }else{
        println!("err unknown broker");
    }
}
pub fn listen_ws_depth(task: TASK, broker: String) {
    let url = get_url(task, &broker, "ethusdt".to_string());
    println!("listen url {} {}", broker, url);
    if broker == "binance" {

            match connect(url.to_string(), |out| binance::WSDepthClient { out: out }) {     Ok(c) => {     println!("connected");       }      Err(err) => { println!("WS Cannot connect {} {}", broker, url) } }
    }else{
        println!("err unknown broker");
    }
}

fn get_url(task: TASK, broker: &String, symbol: String) -> String {
    let mut r = "".to_string();
    match task {
        TASK::HTTP_BIDASK => {
            if broker == "binance" {
                r = binance::URL_HTTP_BIDASK.to_string();
            } else if broker == "hitbtc" {
                r = hitbtc::URL_HTTP_BIDASK.to_string();
            } else if broker == "kraken" {
                r = kraken::URL_HTTP_BIDASK.to_string();
            } else if broker == "kucoin" {
                r = kucoin::URL_HTTP_BIDASK.to_string();
            } else if broker == "cryptopia" {
                r = cryptopia::URL_HTTP_BIDASK.to_string();
            } else if broker == "bitfinex" {
                r = bitfinex::URL_HTTP_BIDASK.to_string();
            }
        }
        TASK::HTTP_PRICE => {
            if broker == "binance" {
                r = binance::URL_HTTP_PRICE.to_string();
            }
        }
        TASK::HTTP_DEPTH => {
            if broker == "binance" {
                r = binance::URL_HTTP_PRICE.to_string();
            }
        }
        TASK::WS_TICK => {
            if broker == "binance" {
                r = binance::URL_WS_TICK.to_string();
                r = str::replace(&r, "XXX", &symbol)
            }
        }
        TASK::WS_DEPTH => {
            if broker == "binance" {
                r = binance::URL_WS_DEPTH.to_string();
                r = str::replace(&r, "XXX", &symbol)
            }
        }
        _ => {}
    }
    r
}
