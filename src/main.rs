extern crate iron;
extern crate time;
extern crate router;
extern crate chrono;
extern crate persistent;

use iron::prelude::*;
use iron::{BeforeMiddleware, AfterMiddleware, typemap};
use time::precise_time_ns;
use std::thread;
use persistent::Write;
use iron::status;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use router::{Router, NoRoute};
use chrono::prelude::*;
use chrono::offset::LocalResult;


use std::collections::HashMap;

use std::rc::Rc;
use std::cell::Cell;


struct ResponseTime;

impl typemap::Key for ResponseTime { type Value = u64; }

//thread_local!(static CACHE :HashMap<String,String>=HashMap::new());

struct Custom404;

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
    //HTTP
    let mut router = Router::new();
    router.get("/", handler_simple, "index");
    router.get("/:query", handler_query_bars, "query");
    router.get("/favicon.ico", handler_favicon, "favicon");
    router.get("/:broker/:pair/:interval/:limit", handler_query_bars, "data");
    let mut chain = Chain::new(router);
    chain.link_before(ResponseTime);
    chain.link_after(ResponseTime);
    chain.link_after(Custom404);
    //chain.link(Write::<HitCounter>::both(0));
    static http_port: i32 = 3000;
    if let Ok(server) = Iron::new(chain).http("localhost:3000") {
        println!("HTTP server listening on {}", http_port);
    } else {
        println!("HTTP server could not connect on {}", http_port);
    }
}


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
                    /*let ts = line[0];
                    let o = line[1];
                    let h = line[2];
                    let l = line[3];
                    let c = line[4];
                    let v = line[5];*/
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
//  CACHE.insert(url.to_string(), res);
        Ok(Response::with((status::Ok, res)))
    } else {
        Ok(Response::with((status::Ok)))
    }
//}
//}
}