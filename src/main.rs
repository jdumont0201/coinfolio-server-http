extern crate iron;
extern crate time;
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

struct ResponseTime;

impl typemap::Key for ResponseTime { type Value = u64; }

struct Custom404;

pub struct Data {
    bid: Option<String>,
    ask: Option<String>,
    last: Option<String>,
}

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
    let mut bidask: BidaskRegistry = Arc::new(Mutex::new(Some(ae)));

    let mut aet: HashMap<String, String> = HashMap::new();
    aet.insert("binance".to_string(), "".to_string());
    aet.insert("hitbtc".to_string(), "".to_string());
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
        sched.add(job_scheduler::Job::new("1/10 * * * * *".parse().unwrap(), || {
            println!("---------------- refresh --------------------");
            refresh_bidask("binance".to_string(), &bidask, &bidasktxt);
            refresh_bidask("hitbtc".to_string(),&bidask,&bidasktxt);

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
        } else {}
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
            } else {
            }
        } else {}
    }
}

//updates the arc mutex
fn update_data(broker: &String, mut persistent: &mut HashMap<String, Data>, fetched: &HashMap<String, Data>) {
    for (symbol, ref data) in fetched.iter() {
        let mut insert: bool = false;
        match persistent.get_mut(symbol) {
            Some(ref mut d) => {
                if data.last.is_some() {  d.last = data.last.clone(); }
                if data.ask.is_some() {  d.ask = data.ask.clone(); }
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
    let mut st = "{".to_string();
    let mut first = true;
    for (symbol, data) in hm.iter() {
        let bid: String;
        let ask: String;
        let last: String;
        match data.bid {
            Some(ref b) => { bid = b.to_string() }
            None => bid = "".to_string()
        }
        match data.ask {
            Some(ref b) => { ask = b.to_string() }
            None => ask = "".to_string()
        }
        match data.last {
            Some(ref b) => { last = b.to_string() }
            None => last = "".to_string()
        }
        if first {
            st = format!("{}\"{}\":{{\"bid\":{},\"ask\":{},\"last\":{}}}", st, symbol, bid, ask, last);
        } else {
            st = format!("{},\"{}\":{{\"bid\":{},\"ask\":{},\"last\":{}}}", st, symbol, bid, ask, last);
        }
        first = false;
    }
    format!("{}}}", st)
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
                None => {}
            }
        } else {}
    }
    Ok(Response::with((status::Ok, val)))
}

#[derive(Serialize, Deserialize)]
struct binance_bidask {
    symbol: String,
    bidPrice: String,
    bidQty: String,
    askPrice: String,
    askQty: String,
}

#[derive(Serialize, Deserialize)]
struct binance_price {
    symbol: String,
    price: String,
}


#[derive(Serialize, Deserialize)]
struct hitbtc_bidask {
    ask: String,
    bid: Option<String>,
    last: String,
    open: String,
    low: String,
    high: String,
    volume: String,
    volumeQuote: String,
    timestamp: String,
    symbol: String,
}


mod Universal {
    use reqwest;
    use std::collections::HashMap;
    use Data;

    fn getGeneric_hashmap(task: String, broker: String, text: String) -> HashMap<String, Data> {
        let mut r = HashMap::new();
        if task == "bidask" {
            if broker == "binance" {
                let bs: Vec<super::binance_bidask> = super::serde_json::from_str(&text).unwrap();
                for row in bs {
                    r.insert(row.symbol, Data { bid: Some(row.bidPrice), ask: Some(row.askPrice), last: None });
                }
            } else if broker == "hitbtc" {
                //println!("format {} {} ",broker,text);
                let bs: Vec<super::hitbtc_bidask> = super::serde_json::from_str(&text).unwrap();
                for row in bs {

                    r.insert(row.symbol, Data { bid: row.bid, ask: Some(row.ask), last: Some(row.last) });
                }
            }
        } else if task == "price" {
            if broker == "binance" {
                let bs: Vec<super::binance_price> = super::serde_json::from_str(&text).unwrap();
                for row in bs {
                    r.insert(row.symbol, Data { bid: None, ask: None, last: Some(row.price) });
                }
            }
        }
        r
    }

    pub fn fetch_bidask(broker: &String) -> HashMap<String, Data> {
        let url = get_url("bidask".to_string(), broker);
        let mut result: HashMap<String, Data>;
        if let Ok(mut res) = reqwest::get(&url) {
            let getres = match res.text() {
                Ok(val) => {
                    let v = getGeneric_hashmap("bidask".to_string(), broker.to_string(), val);
                    result = v;
                }
                Err(err) => {
                    println!("[GET_BIDASK] err");
                    result = HashMap::new();
                }
            };
        } else {
            result = HashMap::new();
        }
        result
    }

    pub fn fetch_price(broker: &String) -> HashMap<String, Data> {
        let url = get_url("price".to_string(), &broker);

        let mut result: HashMap<String, Data>;
        if let Ok(mut res) = reqwest::get(&url) {
            let getres = match res.text() {
                Ok(val) => {

                    let v = getGeneric_hashmap("price".to_string(), broker.to_string(), val);

                    result = v;
                }
                Err(err) => {
                    println!("[GET_PRICE] err");
                    result = HashMap::new();
                }
            };
        } else {
            result = HashMap::new();
        }
        result
    }

    fn get_url(task: String, broker: &String) -> String {
        let mut r = "".to_string();
        if task == "bidask" {
            if broker == "binance" {
                r = "https://api.binance.com/api/v1/ticker/bookTicker".to_string();
            } else if broker == "hitbtc" {
                r = "https://api.hitbtc.com/api/2/public/ticker".to_string();
            }
        } else if task == "price" {
            if broker == "binance" {
                r = "https://api.binance.com/api/v3/ticker/price".to_string();
            } else if broker == "hitbtc" {
                r = "https://api.hitbtc.com/api/2/public/ticker".to_string();
            }
        }
        r
    }
}

fn handler_simple(_req: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, "Up")))
}

fn handler_favicon(_req: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, "Favicon")))
}