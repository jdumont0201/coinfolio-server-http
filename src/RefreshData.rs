use std::error::Error;
use std;
use DataRegistry;
use TextRegistry;
use Universal::{Data, DepthData, RegistryData};
use Universal;

use std::collections::HashMap;
use Brokers::{BROKER, getKey};


//opens the shared data structure for updating bidask
pub fn refresh_bidask(broker: BROKER, mut R: &DataRegistry, RT: &TextRegistry) {
    let key = getKey(broker);
    let fetched = Universal::fetch_bidask(broker);
    let RB = R.get(&key).unwrap();
    if let Ok(mut hm) = RB.write() {
        let mut r: &mut HashMap<String, RegistryData> = &mut *hm;

        update_bidasklast_data(broker, r, &fetched);
        let text = hm_to_text(r);
        update_bidasktext(broker, text, RT);
    } else { println!("err read hashmap val for {}", broker) }
}

//opens the shared data structure for updating price
pub fn refresh_price(broker: BROKER, mut R: &DataRegistry, RT: &TextRegistry) {
    let key = getKey(broker);
    let fetched = Universal::fetch_price(broker);
    let RB = R.get(&key).unwrap();
    if let Ok(mut hm) = RB.write() {
        let mut r: &mut HashMap<String, RegistryData> = &mut *hm;
        update_bidasklast_data(broker, r, &fetched);
        let text = hm_to_text(r);
        update_bidasktext(broker, text, RT);
    } else { println!("err read hashmap val for {}", broker) }
}

pub fn refresh_depth(broker: BROKER, mut R: &DataRegistry, symbol: String, data: DepthData) {
    let key = getKey(broker);
    let RB = R.get(&key).unwrap();
    if let Ok(mut hm) = RB.write() {
        let mut r: &mut HashMap<String, RegistryData> = &mut *hm;
        write_depth_data(broker, r, symbol.to_uppercase(), data)
    } else { println!("err cannot open option bidask {}", broker) }
}

//inserts fresh data into the shared structure content
pub fn write_bidasklast_data(broker: BROKER, mut persistent: &mut HashMap<String, RegistryData>, fetched: &HashMap<String, Data>) {
    for (symbol, ref data) in fetched.iter() {
        let mut insert: bool = false;
        match persistent.get_mut(symbol) {
            Some(ref mut d) => {
                if data.last.is_some() { d.last = data.last.clone(); }
                if data.ask.is_some() { d.ask = data.ask.clone(); }
                if data.bid.is_some() { d.bid = data.bid.clone(); }
            }
            None => {
                insert = true;
            }
        }
        if insert {
            persistent.insert(symbol.to_string(), RegistryData { last: data.last.clone(), ask: data.ask.clone(), bid: data.bid.clone(), asks: Some(vec![]), bids: Some(vec![]) });
        }
    }
}

//inserts fresh data into the shared structure content
pub fn write_depth_data(broker: BROKER, mut persistent: &mut HashMap<String, RegistryData>, symbol: String, data: DepthData) {
    let mut insert: bool = false;
    match persistent.get_mut(&symbol) {
        Some(ref mut d) => {
            if data.bids.is_some() { d.bids = data.bids.clone(); } else { println!("some err"); }
            if data.asks.is_some() { d.asks = data.asks.clone(); } else { println!("some err"); }
        }
        None => {
            insert = true;
        }
    }
    if insert {
        persistent.insert(symbol.to_string().to_uppercase(), RegistryData { last: None, ask: None, bid: None, asks: Some(vec![]), bids: Some(vec![]) });
    }
}
//inserts fresh data into the shared structure content
pub fn write_bidasktext(broker: BROKER, text: String, RT: &TextRegistry) {
    let key = getKey(broker);
    let RB = RT.get(&key).unwrap();
    if let Ok(mut hm) = RB.write() {
        *hm = text;
    }
}
//stringifies the Data Hashmap
pub fn hm_to_text(hm: &HashMap<String, RegistryData>) -> String {
    //println!("hm");
    let mut st = "{".to_string();
    let mut first = true;
    for (symbol, data) in hm.iter() {
        //println!("{} {}",bids,asks);
        let sti = hmi_to_text(symbol.to_string(), data, true);
        if first {
            st = format!("{}{}", st, sti);
        } else {
            st = format!("{},{}", st, sti);
        }
        first = false;
    }
    //println!("hmd");
    //println!("hmd {}",st);
    format!("{}}}", st)
}

//stringifies the Data Hashmap
pub fn hmi_to_text(symbol: String, data: &RegistryData, showSymbol: bool) -> String {
    let bid: String;
    let ask: String;
    let last: String;
    let bids: String;
    let asks: String;
    match data.bid {
        Some(ref b) => { bid = format!("\"{}\"", b.to_string()); }
        None => { bid = "null".to_string(); }
    }
    match data.ask {
        Some(ref b) => { ask = format!("\"{}\"", b.to_string()); }
        None => { ask = "null".to_string(); }
    }
    match data.bids {
        Some(ref b) => {
            bids = format!("{:?}", b);
        }
        None => { bids = "null".to_string(); }
    }
    match data.asks {
        Some(ref b) => { asks = format!("{:?}", b) }
        None => { asks = "null".to_string(); }
    }
    match data.last {
        Some(ref b) => { last = format!("\"{}\"", b.to_string()); }
        None => { last = "null".to_string(); }
    }
    //println!("{} {}",bids,asks);
    if showSymbol {
        format!("\"{}\":{{\"bid\":{},\"ask\":{},\"last\":{},\"bids\":{},\"asks\":{}}}", symbol, bid, ask, last, bids, asks)
    } else {
        format!("{{\"bid\":{},\"ask\":{},\"last\":{},\"bids\":{},\"asks\":{}}}", bid, ask, last, bids, asks)
    }
}
