use std::error::Error;
use debug;
use std;
use std::thread;
use Brokers::{BROKER, getKey, getEnum, TASK, BROKERS};
use chrono::prelude::*;
use types::{DataRegistry, TextRegistry, DictRegistry, OrderbookSide, BidaskRegistry, BidaskReadOnlyRegistry, BidaskTextRegistry};
use Universal::{Data, Universal_Orderbook, Universal_Orderbook_in, RegistryData};
use dictionary::Dictionary;
use Universal;
use job_scheduler;
use routes::hm_to_text;
use std::collections::HashMap;
use dictionary;
use write;
use update;
use dictionary::infrasupraToUniPair;


//opens the shared data structure for updating bidask
pub fn fetch_and_write_bidask(broker: BROKER, R: &DataRegistry, RT: &TextRegistry) {
    let key = getKey(broker);
    let fetched = Universal::fetch_bidask(broker);
    let RB = R.get(&key).unwrap();
    if let Ok(mut hm) = RB.write() {
        let mut r: &mut HashMap<String, RegistryData> = &mut *hm;
        write::write_bidasklast_data(broker, r, &fetched);
        let text = hm_to_text(r);
        write::write_hm(broker, text, RT);
    } else { println!("err read hashmap val for {}", broker) }
}

//opens the shared data structure for updating bidask
pub fn fetch_and_write_depth(broker: BROKER, supra: String, infra: String, R: &DataRegistry, RT: &TextRegistry, DICT: &DictRegistry) {
    let key = getKey(broker);
    let symbol = infrasupraToUniPair(&supra, &infra);
    let rawnameopt = dictionary::read_rawname(broker, supra.to_string(), infra.to_string(), DICT);
    if rawnameopt.is_some() {
        let rawname = rawnameopt.unwrap();
        //println!("rawname {}",rawname);
        let fetched: Universal_Orderbook = Universal::fetch_depth(broker, &infra, &supra, DICT);
        let RB = R.get(&key).unwrap();
        if let Ok(mut hm) = RB.write() {
            let mut r: &mut HashMap<String, RegistryData> = &mut *hm;
            write::write_http_depth_data(broker, r, fetched, rawname);
            let text = hm_to_text(r);
            write::write_hm(broker, text, RT);
        } else {
            debug::err(format!("fetch_and_write_depth cant read hashmap val for {}", broker))
        }
    } else {
        debug::err(format!("fetch_and_write_depth no rawname for {} {}{}", broker, infra, supra));
    }
}
//opens the shared data structure for updating price
pub fn fetch_and_write_price(broker: BROKER, R: &DataRegistry, RT: &TextRegistry) {
    let key = getKey(broker);
    let fetched = Universal::fetch_price(broker);
    let RB = R.get(&key).unwrap();
    if let Ok(mut hm) = RB.write() {
        let mut r: &mut HashMap<String, RegistryData> = &mut *hm;
        write::write_bidasklast_data(broker, r, &fetched);
        let text = hm_to_text(r);
        write::write_hm(broker, text, RT);
    } else { println!("err read hashmap val for {}", broker) }
}


