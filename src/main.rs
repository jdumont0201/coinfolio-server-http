#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_must_use)]
#![warn(unused_mut)]
#[allow(unused_imports)]
#[allow(dead_code)]


//LOAD EXTERNAL MODULES
extern crate base64;
extern crate iron;
extern crate ws;
extern crate time;
extern crate hyper;
extern crate hmac;
extern crate sha2;
extern crate router;
extern crate chrono;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate reqwest;
extern crate job_scheduler;
extern crate colored;


//LOAD CUSTOM MODULES
mod Universal;
mod routes;
mod Brokers;
mod arbitrage;
mod dictionary;
mod commissions;
mod middlewares;
mod fetch;
mod update;
mod write;
mod debug;
mod ws_server;
mod types;

use types::{DataRegistry, TextRegistry, DictRegistry,OrderbookSide,BidaskRegistry, BidaskReadOnlyRegistry, BidaskTextRegistry};
//SPECIFY NAMESPACES
use sha2::Sha256;
use colored::*;
use hmac::{Hmac, Mac};
use base64::{encode, decode};

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

    let DR:DictRegistry=Arc::new(RwLock::new(DICTIONARY.clone()));

    //some clones to feed the threads...
    let RT2 = RT.clone();
    let RT3 = RT.clone();
    let R2 = R.clone();
    let R3 = R.clone();
    let R4 = R.clone();
    let R6 = R.clone();
    let R7 = R.clone();
    let R8 = R.clone();
    let R9 = R.clone();
    let R5 = R.clone();
    let registry5 = R.clone();
    let RT4 = RT.clone();

    let DICT=DR.clone();
    children.push(thread::spawn(move || {
        routes::start_http_server(&RT2,&R2,&DICT);
    }));

    //"update data" threads
    let DICT=DR.clone();
    children.push(thread::spawn(move || {
        fetch::start_datarefresh_thread(&R5, &RT3,&DR);
    }));

    children.push(thread::spawn(move || {    Universal::listen_ws_depth(TASK::WS_DEPTH, BROKER::BINANCE,"ETHUSDT".to_string(),&R8);   }));
    children.push(thread::spawn(move || {    Universal::listen_ws_depth(TASK::WS_DEPTH, BROKER::HITBTC,"ETHUSD".to_string(),&R7); }));
    children.push(thread::spawn(move || {    Universal::listen_ws_depth(TASK::WS_DEPTH, BROKER::BITFINEX,"tETHUSD".to_string(),&R9); }));


    children.push(thread::spawn(move || {
        thread::sleep(std::time::Duration::new(1, 0));
        start_websocket_server();
    }));



    //stay open while threads run
    for child in children {
        let _ = child.join();
    }
}



fn start_websocket_server(){

    ws::listen("127.0.0.1:3012", |out| { ws_server::Server { out: out } }).unwrap()
}