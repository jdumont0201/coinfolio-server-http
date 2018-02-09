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
mod Serve;
mod Refresh;
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
    ae.insert("cryptopia".to_string(), HashMap::new());
    ae.insert("bitfinex".to_string(), HashMap::new());
    let mut bidask: BidaskRegistry = Arc::new(Mutex::new(Some(ae)));

    let mut aet: HashMap<String, String> = HashMap::new();
    aet.insert("binance".to_string(), "".to_string());
    aet.insert("hitbtc".to_string(), "".to_string());
    aet.insert("kucoin".to_string(), "".to_string());
    aet.insert("kraken".to_string(), "".to_string());
    aet.insert("cryptopia".to_string(), "".to_string());
    aet.insert("bitfinex".to_string(), "".to_string());
    let mut bidasktxt: BidaskTextRegistry = Arc::new(Mutex::new(Some(aet)));

    let bidask2 = bidask.clone();
    let bidaskt2 = bidasktxt.clone();
    children.push(thread::spawn(move || {
        //HTTP
        println!("Coinamics Server HTTP");
        let mut router = Router::new();
        router.get("/", Serve::handler_simple, "index");
        router.get("/favicon.ico", Serve::handler_favicon, "favicon");
        let bidask3 = bidaskt2.clone();
        router.get("/public/:broker/bidask", move |request: &mut Request| Serve::get_bidask(request, &bidask3), "ticker");
        router.get("/public/:broker/depth/:pair", move |request: &mut Request| Serve::get_depth(request), "depth");
        let mut chain = Chain::new(router);
        chain.link_before(ResponseTime);
        chain.link_after(ResponseTime);
        chain.link_after(Custom404);
        static HTTP_PORT: i32 = 8080;
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
            Refresh::refresh_bidask("binance".to_string(), &bidask, &bidasktxt);
            Refresh::refresh_bidask("hitbtc".to_string(), &bidask, &bidasktxt);
            Refresh::refresh_bidask("kraken".to_string(), &bidask, &bidasktxt);
            Refresh::refresh_bidask("kucoin".to_string(), &bidask, &bidasktxt);
            Refresh::refresh_bidask("cryptopia".to_string(), &bidask, &bidasktxt);
            Refresh::refresh_bidask("bitfinex".to_string(), &bidask, &bidasktxt);

            thread::sleep(std::time::Duration::new(2, 0));
            Refresh::refresh_price("binance".to_string(), &bidask, &bidasktxt);
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
