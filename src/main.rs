//LOAD EXTERNAL MODULES
extern crate iron;
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

//SPECIFY NAMESPACES
use iron::prelude::*;
use iron::{BeforeMiddleware, AfterMiddleware, typemap};
use time::precise_time_ns;
use std::thread;
use iron::status;
use router::{Router, NoRoute};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use Universal::fetch_bidask;
use Universal::Data;
use Universal::DepthData;
use chrono::prelude::*;
use time::Duration;

//CUSTOM MODULES
mod Universal;
mod Serve;
mod Refresh;
mod middlewares;

//TYPES FOR SHARED STRUCTURES ACROSS THREADS
type BidaskRegistry = Arc<Mutex<Option<HashMap<String, HashMap<String, Data>>>>>;
type BidaskTextRegistry = Arc<Mutex<Option<HashMap<String, String>>>>;

static BROKERS: &'static [&str] = &["binance", "hitbtc", "kucoin", "kraken", "cryptopia", "bitfinex"];

//MAIN
fn main() {
    //THREADS VECTOR
    let mut children = vec![];

    //STRUCTURES SHARED ACROSS THREADS
    //HASHMAP OF DATA  broker -> Data
    let mut ae: HashMap<String, HashMap<String, Data>> = HashMap::new();
    for i in 0..BROKERS.len() {
        ae.insert(BROKERS[i].to_string(), HashMap::new());
    }
    let mut data_registry: BidaskRegistry = Arc::new(Mutex::new(Some(ae)));

    //HASHMAP OF DATA  broker -> formatted string ready to serve through http
    let mut aet: HashMap<String, String> = HashMap::new();
    for i in 0..BROKERS.len() {
        aet.insert(BROKERS[i].to_string(), "".to_string());
    }
    let mut text_registry: BidaskTextRegistry = Arc::new(Mutex::new(Some(aet)));

    //"http server" thread
    let text_registry2 = text_registry.clone();
    children.push(thread::spawn(move || {
        start_http_server(&text_registry2);
    }));

    //"update data" threads
    children.push(thread::spawn(move || {
        start_datarefresh_thread(&data_registry, &text_registry);
    }));

    //keep open while threads run
    for child in children {
        let _ = child.join();
    }
}


fn start_http_server(text_registry: &BidaskTextRegistry) {
    println!("Coinamics Server HTTP");
    //create routes
    let mut router = Router::new();
    router.get("/", Serve::handler_simple, "index");
    router.get("/favicon.ico", Serve::handler_favicon, "favicon");
    let bidask3 = text_registry.clone();
    router.get("/public/:broker/bidask", move |request: &mut Request| Serve::get_bidask(request, &bidask3), "ticker");
    router.get("/public/:broker/depth/:pair", move |request: &mut Request| Serve::get_depth(request), "depth");

    //add middlewares
    let mut chain = Chain::new(router);
    chain.link_before(middlewares::ResponseTime);
    chain.link_after(middlewares::ResponseTime);
    chain.link_after(middlewares::Custom404);

    //listen
    static HTTP_PORT: i32 = 8080;
    let address = "0.0.0.0:8080";
    if let Ok(server) = Iron::new(chain).http(address) {
        println!("HTTP server listening on {}", address);
    } else {
        println!("HTTP server could not connect on {}", address);
    }
}

fn start_datarefresh_thread(data_registry: &BidaskRegistry, text_registry: &BidaskTextRegistry) {
    println!("update data thread");
    let mut sched = job_scheduler::JobScheduler::new();
    sched.add(job_scheduler::Job::new("1/2 * * * * *".parse().unwrap(), || {
        let dt = Local::now();
        println!("{:?}", dt);
        for i in 0..BROKERS.len() {
            let bidask2 = data_registry.clone();
            let bidasktxt2 = text_registry.clone();
            thread::spawn(move || { Refresh::refresh_bidask(BROKERS[i].to_string(), &bidask2, &bidasktxt2); });
        }


        thread::sleep(std::time::Duration::new(2, 0));
        Refresh::refresh_price("binance".to_string(), data_registry, text_registry);
    }));
    loop {
        sched.tick();
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}