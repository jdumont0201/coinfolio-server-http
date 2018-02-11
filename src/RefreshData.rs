use std::error::Error;
use std;
use DataRegistry;
use TextRegistry;
use Universal::{Data, Universal_DepthData, Universal_DepthData_in, RegistryData};
use Universal;
use ServeHTTP::hm_to_text;
use std::collections::HashMap;
use Brokers::{BROKER, getKey};


//opens the shared data structure for updating bidask
pub fn fetch_and_write_bidask(broker: BROKER, R: &DataRegistry, RT: &TextRegistry) {
    let key = getKey(broker);
    let fetched = Universal::fetch_bidask(broker);
    let RB = R.get(&key).unwrap();
    if let Ok(mut hm) = RB.write() {
        let mut r: &mut HashMap<String, RegistryData> = &mut *hm;

        write_bidasklast_data(broker, r, &fetched);
        let text = hm_to_text(r);
        write_bidasktext(broker, text, RT);
    } else { println!("err read hashmap val for {}", broker) }
}

//opens the shared data structure for updating price
pub fn fetch_and_write_price(broker: BROKER, R: &DataRegistry, RT: &TextRegistry) {
    let key = getKey(broker);
    let fetched = Universal::fetch_price(broker);
    let RB = R.get(&key).unwrap();
    if let Ok(mut hm) = RB.write() {
        let mut r: &mut HashMap<String, RegistryData> = &mut *hm;
        write_bidasklast_data(broker, r, &fetched);
        let text = hm_to_text(r);
        write_bidasktext(broker, text, RT);
    } else { println!("err read hashmap val for {}", broker) }
}

pub fn refresh_price(broker: BROKER, R: &DataRegistry, symbol: String, data: Data) {
    let key = getKey(broker);
    let RB = R.get(&key).unwrap();
    if let Ok(mut hm) = RB.write() {
        let mut r: &mut HashMap<String, RegistryData> = &mut *hm;
        write_bidasklast_data_item(broker, r, symbol.to_uppercase(), &data)
    } else { println!("err cannot open option bidask {}", broker) }
}

//inserts fresh data into the shared structure content
pub fn write_bidasklast_data_item(broker: BROKER, persistent: &mut HashMap<String, RegistryData>, symbol: String, data: &Data) {
    let mut insert: bool = false;
    match persistent.get_mut(&symbol) {
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
        persistent.insert(symbol.to_string(), RegistryData { last: data.last.clone(), ask: data.ask.clone(), bid: data.bid.clone(), asks: vec![], bids: vec![] });
    }
}

pub fn write_bidasklast_data(broker: BROKER, persistent: &mut HashMap<String, RegistryData>, fetched: &HashMap<String, Data>) {
    for (symbol, ref data) in fetched.iter() {
        write_bidasklast_data_item(broker, persistent, symbol.to_string(), data);
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


//
// DEPTH
//

//replaces all depth data
pub fn snapshot_depth(broker: BROKER, R: &DataRegistry, symbol: String, data: Universal_DepthData) {
    let key = getKey(broker);
    let RB = R.get(&key).unwrap();
    if let Ok(mut hm) = RB.write() {
        let mut r: &mut HashMap<String, RegistryData> = &mut *hm;
        write_depth_data_item(broker, r, symbol.to_uppercase(), data)
    } else { println!("err cannot open option bidask {}", broker) }
}

//updates depth data
pub fn update_depth(broker: BROKER, R: &DataRegistry, symbol: String, data: Universal_DepthData) {
    let key = getKey(broker);
    let RB = R.get(&key).unwrap();
    if let Ok(mut hm) = RB.write() {
        let mut r: &mut HashMap<String, RegistryData> = &mut *hm;
        write_depth_data_item_for_update(broker, r, symbol.to_uppercase(), data)
    } else { println!("err cannot open option bidask {}", broker) }
}


//inserts fresh data into the shared structure content
pub fn write_depth_data_item(broker: BROKER, persistent: &mut HashMap<String, RegistryData>, symbol: String, data: Universal_DepthData) {
    let mut insert: bool = false;
    match persistent.get_mut(&symbol) {
        Some(ref mut d) => {
            //println!("write depth data {} {} found", broker, symbol);
            d.bids = data.bids.clone();
            d.asks = data.asks.clone();
        }
        None => {
            insert = true;
        }
    }
    if insert {
        println!("write depth data {} {} insert {:?}", broker, symbol.to_string().to_uppercase(), data.asks);
        persistent.insert(symbol.to_string().to_uppercase(), RegistryData { last: None, ask: None, bid: None, asks: data.bids, bids: data.asks });
    }
}


//updates fresh data into the shared structure content
pub fn write_depth_data_item_for_update(broker: BROKER, persistent: &mut HashMap<String, RegistryData>, pair: String, data: Universal_DepthData) {
    let mut insert: bool = false;
    match persistent.get_mut(&pair) {//if pair in hashmap
        Some(ref mut d) => {
            println!("update has");
            update_or_erase_depth_level(&mut d.bids, data.bids.clone());
            update_or_erase_depth_level(&mut d.asks, data.asks.clone());
        }
        None => {
            insert = true;
        }
    }
    println!("update {:?}", data.bids);
    if insert {//should not be run, snapshot should have been run earlier. just in case...
        persistent.insert(pair.to_string().to_uppercase(), RegistryData { last: None, ask: None, bid: None, asks: data.asks, bids: data.bids });
    }
}

fn update_or_erase_depth_level(persistent: &mut Vec<Universal_DepthData_in>, received: Vec<Universal_DepthData_in>) {
    for ref item in received.iter() {
        if item.size == "0.00" { //delete level
            //println!("delete size {} {}", item.price, item.size);
            let mut jval:Option<usize> = None;
            let mut jindex:usize = 0;
            for jtem in persistent.iter_mut() {
                if jtem.price == item.price {
                    jval=Some(jindex);
                    break;
                }
                jindex=jindex+1;
            }
            if jval.is_some() {
                persistent.remove(jval.unwrap());
            }
        } else {
            for ref mut jtem in persistent.iter_mut() {
                if jtem.price == item.price {
              //      println!("add set size {} {}", item.price, item.size);
                    jtem.size = item.size.clone();
                    break;
                }
            }
        }
    }
}
