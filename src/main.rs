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
mod ServeHTTP;
mod RefreshData;
mod middlewares;
mod Brokers;

//SPECIFY NAMESPACES
use iron::{Chain, Request, Iron};
use iron::{BeforeMiddleware, AfterMiddleware, typemap};
use time::precise_time_ns;
use std::thread;
use iron::status;
use router::{Router, NoRoute};
use std::collections::HashMap;
use std::sync::{RwLock ,Arc, Mutex};
use Universal::fetch_bidask;
use Universal::{Data,DepthData,RegistryData};
use chrono::prelude::*;
use time::Duration;
use Brokers::{BROKER,getKey,getEnum,TASK,BROKERS};

//TYPES FOR SHARED STRUCTURES ACROSS THREADS
type DataRegistry = HashMap<String,Arc<RwLock<HashMap<String, RegistryData>>>>;
type TextRegistry = HashMap<String,Arc<RwLock<HashMap<String, String>>>>;

type BidaskRegistry = Arc<Mutex<Option<HashMap<String, HashMap<String, RegistryData>>>>>;
type BidaskReadOnlyRegistry = Arc<RwLock<Option<HashMap<String, HashMap<String, RegistryData>>>>>;
type BidaskTextRegistry = Arc<Mutex<Option<HashMap<String, String>>>>;



//MAIN
fn main() {
    //THREADS VECTOR
    let mut children = vec![];


    //STRUCTURES SHARED ACROSS THREADS
    let R:DataRegistry= HashMap::new();
    //HASHMAP OF DATA  broker -> Data
    let mut ae: HashMap<String, HashMap<String, RegistryData>> = HashMap::new();
    for i in 0..BROKERS.len() {
        ae.insert(BROKERS[i].to_string(), HashMap::new());
    }
    let mut data_registry: BidaskRegistry = Arc::new(Mutex::new(Some(ae)));
    //let mut data_readonly_registry: BidaskReadOnlyRegistry  = Arc::new(RwLock::new(Some((*data_registry.lock().unwrap()).unwrap())));

    //HASHMAP OF DATA  broker -> formatted string ready to serve through http
    let mut aet: HashMap<String, String> = HashMap::new();
    for i in 0..BROKERS.len() {
        aet.insert(BROKERS[i].to_string(), "".to_string());
    }
    let mut text_registry: BidaskTextRegistry = Arc::new(Mutex::new(Some(aet)));

    //READONLY





    //"http server" thread
    let registry3 = data_registry.clone();
    let registry4 = data_registry.clone();
    let registry5 = data_registry.clone();
    //let roregistry5 = data_readonly_registry.clone();
    let text_registry2 = text_registry.clone();
    children.push(thread::spawn(move || {
        start_http_server(&text_registry2,&registry5);
    }));

    //"update data" threads
    children.push(thread::spawn(move || {
        start_datarefresh_thread(&data_registry, &text_registry);
    }));

    children.push(thread::spawn(move || {
        Universal::listen_ws_depth(TASK::WS_DEPTH, BROKER::BINANCE,"btcusdt".to_string(),&registry3);
    }));
    children.push(thread::spawn(move || {
        Universal::listen_ws_depth(TASK::WS_DEPTH, BROKER::BINANCE,"ethusdt".to_string(),&registry4);
    }));
    //keep open while threads run
    for child in children {
        let _ = child.join();
    }
}




fn start_http_server(text_registry: &BidaskTextRegistry,registry:&BidaskRegistry) {
    println!("Coinamics Server HTTP");
    //create routes
    let mut router = Router::new();
    router.get("/", ServeHTTP::handler_simple, "index");
    router.get("/favicon.ico", ServeHTTP::handler_favicon, "favicon");
    let bidask3 = text_registry.clone();
    router.get("/public/:broker/bidask", move |request: &mut Request| ServeHTTP::get_bidask(request, &bidask3), "ticker");
    let bidask4 = registry.clone();
    router.get("/pair/:pair", move |request: &mut Request| ServeHTTP::get_pair(request, &bidask4), "pair");
    router.get("/public/:broker/depth/:pair", move |request: &mut Request| ServeHTTP::get_depth(request), "depth");
    let reg=registry.clone();
    router.get("/target/:broker/:pair", move |request: &mut Request| ServeHTTP::target(request,&reg), "target");

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
            let e = getEnum(BROKERS[i].to_string()).unwrap();
            thread::spawn(move || { RefreshData::refresh_bidask(e, &bidask2, &bidasktxt2); });
        }
        thread::sleep(std::time::Duration::new(2, 0));
        let e = getEnum("binance".to_string()).unwrap();
        RefreshData::refresh_price(e, data_registry, text_registry);
    }));
    loop {
        sched.tick();
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}