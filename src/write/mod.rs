use std::error::Error;
use debug;
use std;use std::thread;
use Brokers::{BROKER,getKey,getEnum,TASK,BROKERS};
use chrono::prelude::*;
use types::{DataRegistry, TextRegistry, DictRegistry, OrderbookSide, BidaskRegistry, BidaskReadOnlyRegistry, BidaskTextRegistry};
use Universal::{Data, Universal_Orderbook, Universal_Orderbook_in, RegistryData};
use dictionary::Dictionary;
use Universal;
use job_scheduler;
use routes::hm_to_text;
use std::collections::HashMap;
use dictionary;

use dictionary::infrasupraToUniPair;


//inserts fresh data into the shared structure content
pub fn write_bidasklast_data_item(broker: BROKER, persistent: &mut HashMap<String, RegistryData>, pair: String, data: &Data) {
    let pair=pair.to_uppercase();
    let mut insert: bool = false;
    match persistent.get_mut(&pair) {
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
        persistent.insert(pair.to_string(), RegistryData::new(data.bid.clone(), data.ask.clone(), data.last.clone(), Universal_Orderbook { asks: HashMap::new(), bids: HashMap::new() }));
    }
}

pub fn write_bidasklast_data(broker: BROKER, persistent: &mut HashMap<String, RegistryData>, fetched: &HashMap<String, Data>) {
    for (symbol, ref data) in fetched.iter() {
        write_bidasklast_data_item(broker, persistent, symbol.to_string(), data);
    }
}

pub fn write_http_depth_data(broker: BROKER, persistent: &mut HashMap<String, RegistryData>, fetched: Universal_Orderbook, symbol: String) {
    write_depth_data_item(broker, persistent, symbol.to_string(), fetched);
}

//inserts fresh data into the shared structure content
pub fn write_bidasktext(broker: BROKER, text: String, RT: &TextRegistry) {
    write_hm(broker, text, RT)
}

//inserts fresh data into the shared structure content
pub fn write_hm(broker: BROKER, text: String, RT: &TextRegistry) {
    let key = getKey(broker);
    let RB = RT.get(&key).unwrap();
    if let Ok(mut hm) = RB.write() {
        *hm = text;
    }
}


//
// DEPTH
//

//inserts fresh data into the shared structure content
pub fn write_depth_data_item(broker: BROKER, persistent: &mut HashMap<String, RegistryData>, pair: String, data: Universal_Orderbook) {
    let pair=pair.to_uppercase();
    let mut insert: bool = false;
    match persistent.get_mut(&pair) {
        Some(ref mut d) => {
            debug::print_write_depth(broker,&pair,&format!("exists"));
            d.set_bids(data.bids.clone());
            d.set_asks(data.asks.clone());
        }
        None => {
            insert = true;
        }
    }
    if insert {
        debug::print_write_depth(broker,&pair,&format!("insert"));
        persistent.insert(pair.to_string(), RegistryData::new(None, None, None, Universal_Orderbook { asks: data.asks, bids: data.bids }));
    }
}


//updates fresh data into the shared structure content
pub fn write_depth_data_item_for_update(broker: BROKER, persistent: &mut HashMap<String, RegistryData>, pair: String, data: Universal_Orderbook) {
    let pair=pair.to_uppercase();
    let mut insert: bool = false;
    match persistent.get_mut(&pair) {//if pair in hashmap
        Some(ref mut d) => {
            debug::print_write_depth(broker,&pair,&format!("{:?} {:?}",data.bids,data.asks));
            update_or_erase_depth_level(&mut d.get_bids_mut(), data.bids.clone());
            update_or_erase_depth_level(&mut d.get_asks_mut(), data.asks.clone());
        }
        None => {
            insert = true;
        }
    }

    if insert {//should not be run, snapshot should have been run earlier. just in case...
        persistent.insert(pair.to_string(), RegistryData::new(None, None, None, Universal_Orderbook { asks: data.asks, bids: data.bids }));
    }
}

fn update_or_erase_depth_level(persistent: &mut OrderbookSide, received: OrderbookSide) {

    for (received_price, received_size) in received {
        if received_size < 0.00000001 { //delete level
            persistent.remove(&received_price);
        } else {
            let mut val: f64 = 0.;
            match persistent.get(&received_price) {
                Some(persistent_size) => {
                    val = received_size + persistent_size;
                }
                None => {
                    val = received_size;
                }
            }
            persistent.insert(received_price, val);
        }
    }
}
