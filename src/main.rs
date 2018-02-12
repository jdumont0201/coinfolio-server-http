#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

//LOAD EXTERNAL MODULES
extern crate iron;
extern crate ws;
extern crate time;
extern crate hyper;
extern crate router;
extern crate chrono;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate reqwest;
extern crate job_scheduler;

//LOAD CUSTOM MODULES
mod Universal;
mod routes;
mod RefreshData;
mod Brokers;
mod arbitrage;
mod dictionary;
mod middlewares;
mod ws_server;
//SPECIFY NAMESPACES

use iron::{Chain, Request, Iron};
use iron::{BeforeMiddleware, AfterMiddleware, typemap};
use time::precise_time_ns;
use std::thread;
use std::cell::RefCell;
use iron::status;
use router::{Router, NoRoute};
use std::collections::HashMap;
use std::sync::{RwLock ,Arc, Mutex};
use Universal::fetch_bidask;
use Universal::{Data,Universal_Orderbook,RegistryData};
use chrono::prelude::*;
use time::Duration;
use dictionary::{Dictionary,generateReference};
use Brokers::{BROKER,getKey,getEnum,TASK,BROKERS};

//TYPES FOR SHARED STRUCTURES ACROSS THREADS
type DataRegistry = HashMap<String,Arc<RwLock<HashMap<String, RegistryData>>>>;
type TextRegistry = HashMap<String,Arc<RwLock<String>>>;
type DictRegistry = Arc<RwLock<Dictionary>>;
type OrderbookSide = HashMap<String,f64>;
type BidaskRegistry = Arc<Mutex<Option<HashMap<String, HashMap<String, RegistryData>>>>>;
type BidaskReadOnlyRegistry = Arc<RwLock<Option<HashMap<String, HashMap<String, RegistryData>>>>>;
type BidaskTextRegistry = Arc<Mutex<Option<HashMap<String, String>>>>;



//MAIN
fn main() {
    let DICTIONARY=generateReference();

    //THREADS VECTOR
    let mut children = vec![];


    //STRUCTURES SHARED ACROSS THREADS
    let mut R:DataRegistry= HashMap::new();
    let mut RT:TextRegistry= HashMap::new();
    //HASHMAP OF DATA  broker -> Data
    for i in 0..BROKERS.len() {
        let mut aei: HashMap<String, RegistryData> = HashMap::new();
        let mut aeit: String="".to_string();
        R.insert(BROKERS[i].to_string(),Arc::new(RwLock::new(aei)));
        RT.insert(BROKERS[i].to_string(),Arc::new(RwLock::new(aeit)));
    }

    //some clones to feed the threads...
    let RT2 = RT.clone();
    let RT3 = RT.clone();
    let R2 = R.clone();
    let R3 = R.clone();
    let R4 = R.clone();
    let R6 = R.clone();
    let R7 = R.clone();
    let R8 = R.clone();
    let R5 = R.clone();
    let registry5 = R.clone();
    let RT4 = RT.clone();

    children.push(thread::spawn(move || {
        let DD:DictRegistry=Arc::new(RwLock::new(DICTIONARY.clone()));
        routes::start_http_server(&RT2,&R2,&DD);
    }));

    //"update data" threads
    children.push(thread::spawn(move || {
        start_datarefresh_thread(&R5, &RT3);
    }));

    children.push(thread::spawn(move || {    Universal::listen_ws_depth(TASK::WS_DEPTH, BROKER::BINANCE,"btcusdt".to_string(),&R3);   }));
    children.push(thread::spawn(move || {    Universal::listen_ws_depth(TASK::WS_DEPTH, BROKER::HITBTC,"BTCUSD".to_string(),&R6); }));
    children.push(thread::spawn(move || {    Universal::listen_ws_depth(TASK::WS_DEPTH, BROKER::BINANCE,"ethusdt".to_string(),&R8);   }));
    children.push(thread::spawn(move || {    Universal::listen_ws_depth(TASK::WS_DEPTH, BROKER::HITBTC,"ETHUSD".to_string(),&R7); }));


    children.push(thread::spawn(move || {
        thread::sleep(std::time::Duration::new(1, 0));
        start_websocket_server();
    }));



    //stay open while threads run
    for child in children {
        let _ = child.join();
    }
}

fn start_datarefresh_thread(R: &DataRegistry, RT: &TextRegistry) {
    println!("update data thread");
    let mut sched = job_scheduler::JobScheduler::new();
    sched.add(job_scheduler::Job::new("1/2 * * * * *".parse().unwrap(), || {
        let dt = Local::now();
        println!("{:?}", dt);
        for i in 0..BROKERS.len() {
            let R2 = R.clone();
            let RT2 = RT.clone();
            let e = getEnum(BROKERS[i].to_string()).unwrap();
            thread::spawn(move || { RefreshData::fetch_and_write_bidask(e, &R2, &RT2); });
        }
        thread::sleep(std::time::Duration::new(2, 0));
        let e = getEnum("binance".to_string()).unwrap();
        RefreshData::fetch_and_write_price(e, R, RT);
    }));
    loop {
        sched.tick();
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}


fn start_websocket_server(){

    ws::listen("127.0.0.1:3012", |out| { ws_server::Server { out: out } }).unwrap()
}