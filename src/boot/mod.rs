use iron::{Chain, Request, Iron};
use iron::{BeforeMiddleware, AfterMiddleware, typemap};
use time::precise_time_ns;
use types::{DataRegistry, TextRegistry, DictRegistry,OrderbookSide,BidaskRegistry, BidaskReadOnlyRegistry, BidaskTextRegistry};

use std::thread;
use std::cell::RefCell;
use iron::status;
use router::{Router, NoRoute};
use std::collections::HashMap;
use std::sync::{RwLock, Arc, Mutex};
use Universal::fetch_bidask;
use Universal::{Data, Universal_Orderbook, RegistryData};
use chrono::prelude::*;
use time::Duration;
use ws;
use ws_server;
use fetch;
use ws::{listen, connect, Handshake, Handler, Sender, Result as wsResult, Message, CloseCode};
use dictionary::{Dictionary, generateReference};
use Brokers::{BROKER, getKey, getEnum, TASK, BROKERS};
use routes;
use std;
use job_scheduler;
pub fn start_http_server(RT: &TextRegistry, R: &DataRegistry, DICT: &DictRegistry) {
    routes::start_http_server(RT, R, DICT)
}


pub fn start_datarefresh_thread(R: &DataRegistry, RT: &TextRegistry, DICT: &DictRegistry) {

    let mut sched = job_scheduler::JobScheduler::new();
    sched.add(job_scheduler::Job::new("1/2 * * * * *".parse().unwrap(), || {
        println!("{:?}", Local::now());
        //refresh price
        for i in 0..BROKERS.len() {
            let R2 = R.clone();
            let RT2 = RT.clone();
            let e = getEnum(BROKERS[i].to_string()).unwrap();
            thread::spawn(move || { fetch::fetch_and_write_bidask(e, &R2, &RT2); });
        }

        //refresh depth
        let RT3 = RT.clone();
        let R3 = R.clone();
        let D3 = DICT.clone();
/*        //let e = getEnum("kucoin".to_string()).unwrap();
        thread::spawn(move || { fetch::fetch_and_write_depth(BROKER::KUCOIN, "ETH".to_string(), "USD".to_string(), &R3, &RT3, &D3); });
        let RT3 = RT.clone();
        let R3 = R.clone();
        let D3 = DICT.clone();
        thread::spawn(move || { fetch::fetch_and_write_depth(BROKER::KRAKEN, "ETH".to_string(), "USD".to_string(), &R3, &RT3, &D3); });
        thread::sleep(std::time::Duration::new(2, 0));
*/
        //refresh price last field(special binance)
        let e = getEnum("binance".to_string()).unwrap();
        fetch::fetch_and_write_price(e, R, RT);
    }));
    loop {
        sched.tick();
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}


pub fn start_websocket_server() {
    ws::listen("127.0.0.1:3012", |out| { ws_server::Server { out: out } }).unwrap()
}