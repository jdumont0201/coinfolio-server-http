use serde_json;
use reqwest;
use std::collections::HashMap;

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



fn get_generic_hashmap(task: String, broker: String, text: String) -> HashMap<String, Data> {
    if task == "bidask" {
        if broker == "bitfinex" {
            bitfinex::get_bidask(text)
        } else if broker == "binance" {
            binance::get_bidask(text)
        } else if broker == "hitbtc" {
           hitbtc::get_bidask(text)
        } else if broker == "cryptopia" {
            cryptopia::get_bidask(text)
        } else if broker == "kucoin" {
            kucoin::get_bidask(text)
        } else if broker == "kraken" {
            kraken::get_bidask(text)
        }else{
            HashMap::new()
        }
    } else if task == "price" {
        if broker == "binance" {
            binance::get_price(text)
        }else{
            HashMap::new()
        }
    }else{
        HashMap::new()
    }

}

fn get_generic_depth_hashmap(task: String, broker: String, text: String) -> String {
    let mut r: String = "".to_string();
    if task == "depth" {
        if broker == "binance" {
            let text2 = str::replace(&text, ",[]", "");

            r = text2;
        }
    }
    r
}

pub fn fetch_bidask(broker: &String) -> HashMap<String, Data> {
    let url = get_url("bidask".to_string(), broker);
    //println!("fetch bidask {} {}", broker,url);
    let mut result: HashMap<String, Data>;
    if let Ok(mut res) = reqwest::get(&url) {
        let getres = match res.text() {
            Ok(val) => {
                let v = get_generic_hashmap("bidask".to_string(), broker.to_string(), val);
      //          println!("fetch bidask {} : get ok", broker);
                result = v;
            }
            Err(err) => {
                println!("[GET_BIDASK] err");
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
    let url = format!("{}{}", get_url("depth".to_string(), broker), pair);
    let mut result: String;
    if let Ok(mut res) = reqwest::get(&url) {
        let getres = match res.text() {
            Ok(val) => {
                let v = get_generic_depth_hashmap("depth".to_string(), broker.to_string(), val);
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
    let url = get_url("price".to_string(), &broker);

    let mut result: HashMap<String, Data>;
    if let Ok(mut res) = reqwest::get(&url) {
        let getres = match res.text() {
            Ok(val) => {
                let v = get_generic_hashmap("price".to_string(), broker.to_string(), val);
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

fn get_url(task: String, broker: &String) -> String {
    let mut r = "".to_string();
    if task == "bidask" {
        if broker == "binance" {
            r = "https://api.binance.com/api/v1/ticker/bookTicker".to_string();
        } else if broker == "hitbtc" {
            r = "https://api.hitbtc.com/api/2/public/ticker".to_string();
        } else if broker == "kraken" {
            r = "https://api.kraken.com/0/public/Ticker?pair=BCHUSD,BCHXBT,DASHUSD,DASHXBT,EOSXBT,GNOXBT,USDTZUSD,XETCXXBT,XETCZUSD,XETHXXBT,XETHZUSD,XETHZUSD.d,XICNXXBT,XLTCXXBT,XLTCZUSD,XMLNXXBT,XREPXXBT,XXBTZCAD,XXBTZCAD.d,XXBTZUSD,XXBTZUSD.d,XXDGXXBT,XXLMXXBT,XXMRXXBT,XXMRZUSD,XXRPXXBT,XXRPZUSD,XZECXXBT,XZECZUSD".to_string()
        } else if broker == "kucoin" {
            r = "https://api.kucoin.com/v1/open/tick".to_string()
        } else if broker == "cryptopia" {
            r = "https://www.cryptopia.co.nz/api/GetMarkets".to_string()
        } else if broker == "bitfinex" {
            r = "https://api.bitfinex.com/v2/tickers?symbols=tBTCUSD,tLTCUSD,tLTCBTC,tETHUSD,tETHBTC,tETCBTC,tETCUSD,tRRTUSD,tRRTBTC,tZECUSD,tZECBTC,tXMRUSD,tXMRBTC,tDSHUSD,tDSHBTC,tBTCEUR,tXRPUSD,tXRPBTC,tIOTUSD,tIOTBTC,tIOTETH,tEOSUSD,tEOSBTC,tEOSETH,tSANUSD,tSANBTC,tSANETH,tOMGUSD,tOMGBTC,tOMGETH,tBCHUSD,tBCHBTC,tBCHETH,tNEOUSD,tNEOBTC,tNEOETH,tETPUSD,tETPBTC,tETPETH,tQTMUSD,tQTMBTC,tQTMETH,tAVTUSD,tAVTBTC,tAVTETH,tEDOUSD,tEDOBTC,tEDOETH,tBTGUSD,tBTGBTC,tDATUSD,tDATBTC,tDATETH,tQSHUSD,tQSHBTC,tQSHETH,tYYWUSD,tYYWBTC,tYYWETH,tGNTUSD,tGNTBTC,tGNTETH,tSNTUSD,tSNTBTC,tSNTETH,tIOTEUR,tBATUSD,tBATBTC,tBATETH,tMNAUSD,tMNABTC,tMNAETH,tFUNUSD,tFUNBTC,tFUNETH,tZRXUSD,tZRXBTC,tZRXETH,tTNBUSD,tTNBBTC,tTNBETH,tSPKUSD,tSPKBTC,tSPKETH,tTRXUSD,tTRXBTC,tTRXETH,tRCNUSD,tRCNBTC,tRCNETH,tRLCUSD,tRLCBTC,tRLCETH,tAIDUSD,tAIDBTC,tAIDETH,tSNGUSD,tSNGBTC,tSNGETH,tREPUSD,tREPBTC,tREPETH,tELFUSD,tELFBTC,tELFETH".to_string()
        }
    } else if task == "price" {
        if broker == "binance" {
            r = "https://api.binance.com/api/v3/ticker/price".to_string();
        } else if broker == "hitbtc" {
            r = "https://api.hitbtc.com/api/2/public/ticker".to_string();
        }
    } else if task == "depth" {
        if broker == "binance" {
            r = "https://api.binance.com/api/v1/depth?symbol=".to_string()
        }
    }
    r
}
