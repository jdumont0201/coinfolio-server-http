use std::error::Error;
use std;
use DataRegistry;
use TextRegistry;
use OrderbookSide;
use Universal::{Data, Universal_Orderbook, Universal_Orderbook_in, RegistryData};
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
        persistent.insert(symbol.to_string(), RegistryData::new( data.bid.clone(), data.ask.clone(), data.last.clone(), Universal_Orderbook {asks: HashMap::new(), bids: HashMap::new() } ));
        //persistent.insert(symbol.to_string(), RegistryData { last: data.last.clone(), ask: data.ask.clone(), bid: data.bid.clone(), orderbook: Universal_Orderbook {asks: HashMap::new(), bids: HashMap::new() }});
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
            //println!("write depth data {} {} found", broker, symbol);
            d.set_bids(data.bids.clone());
            d.set_asks(data.asks.clone());
        }
        None => {
            insert = true;
        }
    }
    if insert {

        //for (price, size) in data.get_bids().iter() {
            //println!("bids write {}{}",price,size);
        //}
        persistent.insert(pair.to_string(), RegistryData::new(None, None,None, Universal_Orderbook  {  asks: data.asks, bids: data.bids } ));
        //persistent.insert(symbol.to_string().to_uppercase(), RegistryData { last: None, ask: None, bid: None,orderbook:Universal_Orderbook { asks: data.bids, bids: data.asks} });
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
        //persistent.insert(pair.to_string().to_uppercase(), RegistryData { last: None, ask: None, bid: None,orderbook:Universal_Orderbook {  asks: data.asks, bids: data.bids }});
        persistent.insert(pair.to_string(), RegistryData::new(None, None,None, Universal_Orderbook  {  asks: data.asks, bids: data.bids } ));
    }
}

fn update_or_erase_depth_level(persistent: &mut OrderbookSide, received: OrderbookSide) {
    for (received_price, received_size) in received {
        if received_size < 0.00000001 { //delete level
            persistent.remove(&received_price);
        } else {
            let mut val:f64=0.;
            match persistent.get(&received_price) {
                Some(persistent_size)=>{
                    val=received_size+persistent_size;
                },None=>{
                    val=received_size;
                }
            }
            persistent.insert(received_price,val);
        }
    }
}
