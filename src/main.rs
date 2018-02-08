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

use iron::prelude::*;
use iron::{BeforeMiddleware, AfterMiddleware, typemap};
use time::precise_time_ns;
use std::thread;
use iron::status;
use std::fs::File;
use std::io::prelude::*;
use router::{Router, NoRoute};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
mod Universal;
use Universal::fetch_bidask;
use Universal::Data;
use Universal::DepthData;
struct ResponseTime;

impl typemap::Key for ResponseTime { type Value = u64; }

struct Custom404;



type BidaskRegistry = Arc<Mutex<Option<HashMap<String, HashMap<String, Data>>>>>;
type BidaskTextRegistry = Arc<Mutex<Option<HashMap<String, String>>>>;

impl BeforeMiddleware for ResponseTime {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        req.extensions.insert::<ResponseTime>(precise_time_ns());
        Ok(())
    }
}

impl AfterMiddleware for ResponseTime {
    fn after(&self, req: &mut Request, res: Response) -> IronResult<Response> {
        let delta = precise_time_ns() - *req.extensions.get::<ResponseTime>().unwrap();
        println!("{} Request took: {} ms", req.url, (delta as f64) / 1000000.0);
        Ok(res)
    }
}

impl AfterMiddleware for Custom404 {
    fn catch(&self, _: &mut Request, err: IronError) -> IronResult<Response> {
        println!("Hitting custom 404 middleware");

        if err.error.is::<NoRoute>() {
            Ok(Response::with((status::NotFound, "404")))
        } else {
            Err(err)
        }
    }
}

fn main() {
    let mut children = vec![];

    let mut ae: HashMap<String, HashMap<String, Data>> = HashMap::new();
    ae.insert("binance".to_string(), HashMap::new());
    ae.insert("hitbtc".to_string(), HashMap::new());
    ae.insert("kucoin".to_string(), HashMap::new());
    ae.insert("kraken".to_string(), HashMap::new());
    ae.insert("cryptonia".to_string(), HashMap::new());
    let mut bidask: BidaskRegistry = Arc::new(Mutex::new(Some(ae)));

    let mut aet: HashMap<String, String> = HashMap::new();
    aet.insert("binance".to_string(), "".to_string());
    aet.insert("hitbtc".to_string(), "".to_string());
    aet.insert("kucoin".to_string(), "".to_string());
    aet.insert("kraken".to_string(), "".to_string());
    aet.insert("cryptonia".to_string(), "".to_string());
    let mut bidasktxt: BidaskTextRegistry = Arc::new(Mutex::new(Some(aet)));

    let bidask2 = bidask.clone();
    let bidaskt2 = bidasktxt.clone();
    children.push(thread::spawn(move || {
        //HTTP
        println!("Coinamics Server HTTP");
        let mut router = Router::new();
        router.get("/", handler_simple, "index");
        router.get("/favicon.ico", handler_favicon, "favicon");
        let bidask3 = bidaskt2.clone();
        router.get("/public/:broker/bidask", move |request: &mut Request| get_bidask(request, &bidask3), "ticker");
        router.get("/public/:broker/depth/:pair", move |request: &mut Request| get_depth(request), "depth");
        let mut chain = Chain::new(router);
        chain.link_before(ResponseTime);
        chain.link_after(ResponseTime);
        chain.link_after(Custom404);
        static http_port: i32 = 8080;
        let address = "0.0.0.0:8080";
        if let Ok(server) = Iron::new(chain).http(address) {
            println!("HTTP server listening on {}", address);
        } else {
            println!("HTTP server could not connect on {}", address);
        }
    }));

    //"update data" thread
    children.push(thread::spawn(move || {
        println!("update data thread");
        let mut sched = job_scheduler::JobScheduler::new();
        sched.add(job_scheduler::Job::new("1/2 * * * * *".parse().unwrap(), || {
            println!("---------------- refresh --------------------");
            refresh_bidask("binance".to_string(), &bidask, &bidasktxt);
            refresh_bidask("hitbtc".to_string(), &bidask, &bidasktxt);
            refresh_bidask("kraken".to_string(), &bidask, &bidasktxt);
            refresh_bidask("kucoin".to_string(), &bidask, &bidasktxt);
            refresh_bidask("cryptonia".to_string(), &bidask, &bidasktxt);

            thread::sleep(std::time::Duration::new(2, 0));
            refresh_price("binance".to_string(), &bidask, &bidasktxt);
        }));
        loop {
            sched.tick();
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    }));
    for child in children {
        let _ = child.join();
    }
}

fn refresh_bidask(broker: String, mut bidask: &BidaskRegistry, bidaskt: &BidaskTextRegistry) {
    println!("refresh {}", broker);
    if let Ok(mut opt) = bidask.lock() {
        if let Some(ref mut hm) = *opt { //open option
            let mut val: Option<&mut HashMap<String, Data>> = hm.get_mut(&broker);
            if let Some(mut vv) = val {
                let fetched = Universal::fetch_bidask(&broker);
                update_data(&broker, vv, &fetched);
                let text = hmToText(vv);

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

fn refresh_price(broker: String, mut bidask: &BidaskRegistry, bidaskt: &BidaskTextRegistry) {
    if let Ok(mut opt) = bidask.lock() {
        if let Some(ref mut hm) = *opt { //open option
            let mut val: Option<&mut HashMap<String, Data>> = hm.get_mut(&broker);
            if let Some(mut vv) = val {
                let fetched = Universal::fetch_price(&broker);
                update_data(&broker, vv, &fetched);
                let text = hmToText(vv);
                update_bidasktext(&broker, text, bidaskt);
                //*vv=hm;
            } else {}
        } else {}
    }
}

//updates the arc mutex
fn update_data(broker: &String, mut persistent: &mut HashMap<String, Data>, fetched: &HashMap<String, Data>) {
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

fn update_bidasktext(broker: &String, text: String, bidaskt: &BidaskTextRegistry) {
    if let Ok(mut opt) = bidaskt.lock() {
        if let Some(ref mut hm) = *opt { //open option
            let mut val: Option<&mut String> = hm.get_mut(broker);
            if let Some(mut vv) = val {
                *vv = text;
            }
        }
    }
}

fn hmToText(hm: &HashMap<String, Data>) -> String {
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

fn get_depth(req: &mut Request) -> IronResult<Response> {
    let ref broker: &str = req.extensions.get::<Router>().unwrap().find("broker").unwrap_or("/");
    let ref pair: &str = req.extensions.get::<Router>().unwrap().find("pair").unwrap_or("/");
    let text= Universal::fetch_depth(&broker.to_string(), &pair.to_string());


    let mut res = Response::with((status::Ok, text));
    res.headers.set(iron::headers::AccessControlAllowOrigin::Any);
    Ok(res)
}
fn get_bidask(req: &mut Request, ticker: &BidaskTextRegistry) -> IronResult<Response> {
    let ref broker: &str = req.extensions.get::<Router>().unwrap().find("broker").unwrap_or("/");
    let key: String = broker.to_string();
    let mut val: String = "".to_string();
    if let Ok(mut opt) = ticker.lock() {
        if let Some(ref mut hm) = *opt { //open option
            let br = broker.to_string();
            match hm.get(&br) {
                Some(op) => {
                    val = op.to_string();
                }
                None => {
                    println!("No match for text broker {}", broker)
                }
            }
        } else {
            println!("Cannot open options {}", broker)
        }
    } else {
        println!("Cannot lock arc {}", broker)
    }

    let mut res = Response::with((status::Ok, val));
    res.headers.set(iron::headers::AccessControlAllowOrigin::Any);
    Ok(res)
}

fn handler_simple(_req: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, "Up")))
}

fn handler_favicon(_req: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, "Favicon")))
}