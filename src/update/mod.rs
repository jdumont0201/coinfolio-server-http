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
use write;
use dictionary::infrasupraToUniPair;

pub fn refresh_price(broker: BROKER, R: &DataRegistry, pair: String, data: Data) {
    let key = getKey(broker);
    let RB = R.get(&key).unwrap();
    if let Ok(mut hm) = RB.write() {
        let mut r: &mut HashMap<String, RegistryData> = &mut *hm;
        write::write_bidasklast_data_item(broker, r, pair.to_uppercase(), &data)
    } else { println!("err cannot open option bidask {}", broker) }
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
        write::write_depth_data_item(broker, r, pair.to_uppercase(), data)
    } else { println!("err cannot open option bidask {}", broker) }
}

//updates depth data
pub fn update_depth(broker: BROKER, R: &DataRegistry, pair: String, data: Universal_Orderbook) {
    let key = getKey(broker);
    let RB = R.get(&key).unwrap();
    if let Ok(mut hm) = RB.write() {
        let mut r: &mut HashMap<String, RegistryData> = &mut *hm;
        write::write_depth_data_item_for_update(broker, r, pair.to_uppercase(), data)
    } else { println!("err cannot open option bidask {}", broker) }
}