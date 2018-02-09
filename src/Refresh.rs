use BidaskRegistry;
use BidaskTextRegistry;
use Universal::Data;
use Universal;
use Universal::DepthData;
use std::collections::HashMap;
pub fn refresh_bidask(broker: String, mut bidask: &BidaskRegistry, bidaskt: &BidaskTextRegistry) {
    println!("refresh {}", broker);
    if let Ok(mut opt) = bidask.lock() {
        if let Some(ref mut hm) = *opt { //open option
            let mut val: Option<&mut HashMap<String, Data>> = hm.get_mut(&broker);
            if let Some(mut vv) = val {
                let fetched = Universal::fetch_bidask(&broker);
                update_data(&broker, vv, &fetched);
                let text = hm_to_text(vv);

                update_bidasktext(&broker, text, bidaskt);
                //*vv=hm;
            }
        } else {
            println!("err cannot open option bidask {}", broker)
        }
    } else {
        println!("err cannot lock arcmutex bidask {}", broker)
    }
}

pub fn refresh_price(broker: String, mut bidask: &BidaskRegistry, bidaskt: &BidaskTextRegistry) {
    if let Ok(mut opt) = bidask.lock() {
        if let Some(ref mut hm) = *opt { //open option
            let mut val: Option<&mut HashMap<String, Data>> = hm.get_mut(&broker);
            if let Some(mut vv) = val {
                let fetched = Universal::fetch_price(&broker);
                update_data(&broker, vv, &fetched);
                let text = hm_to_text(vv);
                update_bidasktext(&broker, text, bidaskt);
                //*vv=hm;
            } else {}
        } else {}
    }
}

//updates the arc mutex
pub fn update_data(broker: &String, mut persistent: &mut HashMap<String, Data>, fetched: &HashMap<String, Data>) {
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
            persistent.insert(symbol.to_string(), Data { last: data.last.clone(), ask: data.ask.clone(), bid: data.bid.clone() });
        }
    }
}

pub fn update_bidasktext(broker: &String, text: String, bidaskt: &BidaskTextRegistry) {
    if let Ok(mut opt) = bidaskt.lock() {
        if let Some(ref mut hm) = *opt { //open option
            let mut val: Option<&mut String> = hm.get_mut(broker);
            if let Some(mut vv) = val {
                *vv = text;
            }
        }
    }
}

pub fn hm_to_text(hm: &HashMap<String, Data>) -> String {
    //println!("hm");
    let mut st = "{".to_string();
    let mut first = true;
    for (symbol, data) in hm.iter() {
        let bid: String;
        let ask: String;
        let last: String;
        match data.bid {
            Some(ref b) => { bid = format!("\"{}\"", b.to_string()); }
            None => { bid = "null".to_string(); }
        }
        match data.ask {
            Some(ref b) => { ask = format!("\"{}\"", b.to_string()); }
            None => { ask = "null".to_string(); }
        }
        match data.last {
            Some(ref b) => { last = format!("\"{}\"", b.to_string()); }
            None => { last = "null".to_string(); }
        }
        if first {
            st = format!("{}\"{}\":{{\"bid\":{},\"ask\":{},\"last\":{}}}", st, symbol, bid, ask, last);
        } else {
            st = format!("{},\"{}\":{{\"bid\":{},\"ask\":{},\"last\":{}}}", st, symbol, bid, ask, last);
        }
        first = false;
    }
    //println!("hmd");
    //println!("hmd {}",st);
    format!("{}}}", st)
}
