use std::error::Error;
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

pub fn start_datarefresh_thread(R: &DataRegistry, RT: &TextRegistry, DICT: &DictRegistry) {

    let mut sched = job_scheduler::JobScheduler::new();
    sched.add(job_scheduler::Job::new("1/2 * * * * *".parse().unwrap(), || {
        println!("{:?}", Local::now());
        //refresh price
        for i in 0..BROKERS.len() {
            let R2 = R.clone();
            let RT2 = RT.clone();
            let e = getEnum(BROKERS[i].to_string()).unwrap();
            thread::spawn(move || { fetch_and_write_bidask(e, &R2, &RT2); });
        }

        //refresh depth
        let RT3 = RT.clone();
        let R3 = R.clone();
        let D3 = DICT.clone();

        let e = getEnum("kucoin".to_string()).unwrap();
        thread::spawn(move || { fetch_and_write_depth(e, "ETH".to_string(), "USD".to_string(), &R3, &RT3, &D3); });
        thread::sleep(std::time::Duration::new(2, 0));

        //refresh price last field(special binance)
        let e = getEnum("binance".to_string()).unwrap();
        fetch_and_write_price(e, R, RT);
    }));
    loop {
        sched.tick();
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}


//opens the shared data structure for updating bidask
pub fn fetch_and_write_bidask(broker: BROKER, R: &DataRegistry, RT: &TextRegistry) {
    let key = getKey(broker);
    let fetched = Universal::fetch_bidask(broker);
    let RB = R.get(&key).unwrap();
    if let Ok(mut hm) = RB.write() {
        let mut r: &mut HashMap<String, RegistryData> = &mut *hm;
        write_bidasklast_data(broker, r, &fetched);
        let text = hm_to_text(r);
        write_hm(broker, text, RT);
    } else { println!("err read hashmap val for {}", broker) }
}

//opens the shared data structure for updating bidask
pub fn fetch_and_write_depth(broker: BROKER, supra: String, infra: String, R: &DataRegistry, RT: &TextRegistry, DICT: &DictRegistry) {
    let key = getKey(broker);
    let symbol = infrasupraToUniPair(&supra, &infra);
    let rawnameopt = dictionary::read_rawname(broker, supra.to_string(), infra.to_string(), DICT);
    if rawnameopt.is_some() {
        let rawname=rawnameopt.unwrap();
        let fetched: Universal_Orderbook = Universal::fetch_depth(broker, &infra, &supra, DICT);
        let RB = R.get(&key).unwrap();
        if let Ok(mut hm) = RB.write() {
            let mut r: &mut HashMap<String, RegistryData> = &mut *hm;
            write_http_depth_data(broker, r, fetched, rawname);
            let text = hm_to_text(r);
            write_hm(broker, text, RT);
        } else { println!("err read hashmap val for {}", broker) }
    }
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
        write_hm(broker, text, RT);
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
        persistent.insert(symbol.to_string(), RegistryData::new(data.bid.clone(), data.ask.clone(), data.last.clone(), Universal_Orderbook { asks: HashMap::new(), bids: HashMap::new() }));
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

//replaces all depth data
pub fn snapshot_depth(broker: BROKER, R: &DataRegistry, pair: String, data: Universal_Orderbook) {
    let key = getKey(broker);
    let RB = R.get(&key).unwrap();
    if let Ok(mut hm) = RB.write() {
        let mut r: &mut HashMap<String, RegistryData> = &mut *hm;
        write_depth_data_item(broker, r, pair.to_uppercase(), data)
    } else { println!("err cannot open option bidask {}", broker) }
}

//updates depth data
pub fn update_depth(broker: BROKER, R: &DataRegistry, pair: String, data: Universal_Orderbook) {
    let key = getKey(broker);
    let RB = R.get(&key).unwrap();
    if let Ok(mut hm) = RB.write() {
        let mut r: &mut HashMap<String, RegistryData> = &mut *hm;
        write_depth_data_item_for_update(broker, r, pair.to_uppercase(), data)
    } else { println!("err cannot open option bidask {}", broker) }
}


//inserts fresh data into the shared structure content
pub fn write_depth_data_item(broker: BROKER, persistent: &mut HashMap<String, RegistryData>, pair: String, data: Universal_Orderbook) {
    let mut insert: bool = false;
    match persistent.get_mut(&pair) {
        Some(ref mut d) => {
            d.set_bids(data.bids.clone());
            d.set_asks(data.asks.clone());
        }
        None => {
            insert = true;
        }
    }
    if insert {
        persistent.insert(pair.to_string(), RegistryData::new(None, None, None, Universal_Orderbook { asks: data.asks, bids: data.bids }));
    }
}


//updates fresh data into the shared structure content
pub fn write_depth_data_item_for_update(broker: BROKER, persistent: &mut HashMap<String, RegistryData>, pair: String, data: Universal_Orderbook) {
    let mut insert: bool = false;
    match persistent.get_mut(&pair) {//if pair in hashmap
        Some(ref mut d) => {
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
