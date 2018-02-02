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
type BidaskRegistry = Arc<Mutex<Option<HashMap<String, String>>>>;

impl BeforeMiddleware for ResponseTime {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        req.extensions.insert::<ResponseTime>(precise_time_ns());
        Ok(())
    }
}

impl AfterMiddleware for ResponseTime {
    fn after(&self, req: &mut Request, res: Response) -> IronResult<Response> {
        let delta = precise_time_ns() - *req.extensions.get::<ResponseTime>().unwrap();
        println!("{:?} Request took: {} ms", *req, (delta as f64) / 1000000.0);
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
    let mut ae: HashMap<String, String> = HashMap::new();
    ae.insert("binance".to_string(), "".to_string());
    ae.insert("hitbtc".to_string(), "".to_string());
    let mut bidask: BidaskRegistry = Arc::new(Mutex::new(Some(ae)));
    //let mut TICKER: HashMap<String,String> = HashMap::new();
    let bidask2 = bidask.clone();
    children.push(thread::spawn(move || {

        //HTTP
        println!("Coinamics Server HTTP");
        let mut router = Router::new();
        router.get("/", handler_simple, "index");
        router.get("/:query", handler_query_bars, "query");
        router.get("/favicon.ico", handler_favicon, "favicon");
        router.get("/:broker/:pair/:interval/:limit", handler_query_bars, "data");

        let bidask3 = bidask2.clone();
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
        //let mut tick=ticker.clone();
        sched.add(job_scheduler::Job::new("1/10 * * * * *".parse().unwrap(), || {
            println!("refresh bidask");
            refresh_bidask("binance".to_string(),&bidask);
            refresh_bidask("hitbtc".to_string(),&bidask);
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

fn refresh_bidask(broker: String, bidask:&BidaskRegistry) {
    if let Ok(mut opt) = bidask.lock() {
        if let Some(ref mut hm) = *opt { //open option
            println!("refresh bidask {}", broker);
            let mut val: Option<&mut String> = hm.get_mut(&broker);
            if let Some(mut vv) = val {
                let text = Universal::fetch_bidask(broker);

                *vv = text;
            }
        } else {}
    }
}

fn get_bidask(req: &mut Request, ticker: &BidaskRegistry) -> IronResult<Response> {
    println!("bidask");
    let ref broker: &str = req.extensions.get::<Router>().unwrap().find("broker").unwrap_or("/");
    println!("bidask {}", broker);
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

    fn format(task: String, broker: String, text: String) -> String {
        let mut r = "{".to_string();
        let mut first = true;
        if task == "bidask" {
            if broker == "binance" {
                let bs: Vec<super::binance_bidask> = super::serde_json::from_str(&text).unwrap();
                for row in bs {
                    if first {
                        r = format!("{}\"{}\":{{\"bid\":{},\"ask\":{}}}", r, row.symbol, row.bidPrice, row.askPrice);
                    } else {
                        r = format!("{},\"{}\":{{\"bid\":{},\"ask\":{}}}", r, row.symbol, row.bidPrice, row.askPrice);
                    }
                    first = false;
                }
            } else if broker == "hitbtc" {
                println!("format {} {} ",broker,text);
                let bs: Vec<super::hitbtc_bidask> = super::serde_json::from_str(&text).unwrap();
                for row in bs {
                    let bid;
                    match row.bid {
                        Some(b)=>{ bid=b;},
                        None => {bid="".to_string()}
                    };
                    if first {
                        r = format!("{}\"{}\":{{\"bid\":{},\"ask\":{}}}", r, row.symbol, bid, row.ask);
                    } else {
                        r = format!("{},\"{}\":{{\"bid\":{},\"ask\":{},\"p\":{}}}", r, row.symbol, bid, row.ask,row.last);
                    }
                    first = false;
                }
            }
        }
        format!("{}}}", r)
    }

    pub fn fetch_bidask(broker: String) -> String {
        let url = get_url("bidask".to_string(), &broker);
        println!("url {}",url);
        let mut result: String = "".to_string();
        if let Ok(mut res) = reqwest::get(&url) {
            let getres = match res.text() {
                Ok(val) => {
                    let v = format("bidask".to_string(), broker, val);
                    result = v;
                }
                Err(err) => {
                    println!("[GET_TICKER] err");
                }
            };
        } else {}
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
        }else if task == "price" {
            if broker == "binance" {
                r = "https://api.binance.com/api/v3/ticker/price".to_string();
            } else if broker == "hitbtc" {
                r = "https://api.hitbtc.com/api/2/public/ticker".to_string();
            }
        }
        r
    }
}

/*fn updateTicker(broker: String,mut ticker:&mut HashMap<String,String>) {
    let br=broker.clone();
    let data:String=Universal::fetch_ticker(broker);
    ticker[&br]  =data;

}*/

fn handler_simple(_req: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, "Up")))
}

fn handler_favicon(_req: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, "Favicon")))
}

fn get_file_url(base_url: &str, broker: &str, pair: &str, interval: &str) -> String {
    let mut s = base_url.to_string().to_owned();
    s.push_str(broker);
    s.push_str("-");
    s.push_str(interval);
    s.push_str("-");
    s.push_str(pair);
    s.push_str(".csv");
    s
}

fn handler_query_bars(req: &mut Request) -> IronResult<Response> {
//let dt = std::time::Instant::now();
    let ref broker: &str = req.extensions.get::<Router>().unwrap().find("broker").unwrap_or("/");
    let url = &req.url;
    /*match CACHE.get(url) {
        Some(&res) => {
            Ok(Response::with((status::Ok, res)))
        }
        None => {*/
    if broker.len() > 1 {
        let ref interval = req.extensions.get::<Router>().unwrap().find("interval").unwrap_or("/");
        let ref pair = req.extensions.get::<Router>().unwrap().find("pair").unwrap_or("/");
        let ref limit = req.extensions.get::<Router>().unwrap().find("limit").unwrap_or("/");
        let n_results: usize = limit.parse::<usize>().unwrap();
        let filename = get_file_url("../websockets/data/", broker, pair, interval);
        println!("File {}", filename);
        let mut f = File::open(filename).expect("file not found");
        let mut contents = String::new();
        f.read_to_string(&mut contents).expect("something went wrong reading the file");
        let split = contents.split("\n");
        let splitcol: Vec<&str> = split.collect();
//  let dt0 = dt.elapsed();
//let NN=split.count();
        let N: usize = splitcol.len();
        println!("lines {}", N);
        let mut res: String = "{\"error\":\"false\",\"success\":\"true\",\"data\":[".to_string();

        let mut first = true;
        let mut current_line = 0;
        let min: usize = 0;
        let begin = std::cmp::max(min, N - n_results);
        for s in splitcol.iter() {
            if (current_line > begin) {
                let line: Vec<&str> = s.split(",").collect();
                if (line.len() > 1) {
                    if (!first) { res.push_str(","); }
                    res.push_str("{\"ts\":\"");
                    res.push_str(line[0]);
                    res.push_str("\",\"o\":\"");
                    res.push_str(line[1]);
                    res.push_str("\",\"h\":\"");
                    res.push_str(line[2]);
                    res.push_str("\",\"l\":\"");
                    res.push_str(line[3]);
                    res.push_str("\",\"c\":\"");
                    res.push_str(line[4]);
                    res.push_str("\",\"v\":\"");
                    res.push_str(line[5]);
                    res.push_str("\"}");
                    first = false;
                }
            }
            current_line = current_line + 1;
        }
        res.push_str("] }");
        println!("Cache {}", url.to_string());
        Ok(Response::with((status::Ok, res)))
    } else {
        Ok(Response::with((status::Ok)))
    }
}