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
use update;
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
        //let e = getEnum("kucoin".to_string()).unwrap();
        thread::spawn(move || { fetch_and_write_depth(BROKER::KUCOIN, "ETH".to_string(), "USD".to_string(), &R3, &RT3, &D3); });
        let RT3 = RT.clone();
        let R3 = R.clone();
        let D3 = DICT.clone();
        thread::spawn(move || { fetch_and_write_depth(BROKER::KRAKEN, "ETH".to_string(), "USD".to_string(), &R3, &RT3, &D3); });
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
        let rawname=rawnameopt.unwrap();
        //println!("rawname {}",rawname);
        let fetched: Universal_Orderbook = Universal::fetch_depth(broker, &infra, &supra, DICT);
        let RB = R.get(&key).unwrap();
        if let Ok(mut hm) = RB.write() {
            let mut r: &mut HashMap<String, RegistryData> = &mut *hm;
            write::write_http_depth_data(broker, r, fetched, rawname);
            let text = hm_to_text(r);
            write::write_hm(broker, text, RT);
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
        write::write_bidasklast_data(broker, r, &fetched);
        let text = hm_to_text(r);
        write::write_hm(broker, text, RT);
    } else { println!("err read hashmap val for {}", broker) }
}


