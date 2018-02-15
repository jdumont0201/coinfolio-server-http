use std::thread;
use std::iter::FromIterator;
use std::collections::HashMap;
use iron::{Request, Response, Chain, IronResult, Iron};
use Universal;
use Universal::{Data, Universal_Orderbook, RegistryData};
use iron::status;
use arbitrage::{recap,approx};
use debug;
use iron;
use fetch;
use std;
use std::cell::RefCell;
use std::sync::RwLock;
use router::{Router, NoRoute};
use types::{DataRegistry, TextRegistry, DictRegistry, OrderbookSide, BidaskRegistry, BidaskReadOnlyRegistry, BidaskTextRegistry};
use Brokers::{BROKER, getEnum, TASK, BROKERS};
use dictionary::Dictionary;
use job_scheduler;
use middlewares;

pub fn start_http_server(RT: &TextRegistry, R: &DataRegistry, DICT: &DictRegistry) {
    println!("Coinamics Server HTTP");
    //create routes
    let mut router = Router::new();
    router.get("/", handler_simple, "index");
    router.get("/favicon.ico", handler_favicon, "favicon");
    let RT2 = RT.clone();
    let RT3 = RT.clone();
    router.get("/exchange/:broker/price", move |request: &mut Request| get_bidask(request, &RT2), "ticker");
    let R2 = R.clone();
    router.get("/pair/:pair", move |request: &mut Request| get_pair(request, &R2), "pair");
    let R2b = R.clone();
    let R2c = R.clone();

    let DD2 = DICT.clone();
    let DD3 = DICT.clone();
    router.get("/task/arbitrage/budget/:budget/supra/:supra/infra/:infra", move |request: &mut Request| get_arbitrage(request, &R2b, &DD2), "infrasupra");
    router.get("/task/approx", move |request: &mut Request| get_arbitrage_approx(request, &R2c, &DD3), "infrasupraapprox");
    router.get("/exchange/:broker/task/depth/symbol/:pair", move |request: &mut Request| get_depth(request), "depth");
    let R3 = R.clone();
    let DD3 = DICT.clone();
    router.get("/task/subscribe/supra/:supra/infra/:infra", move |request: &mut Request| route_target_all(request, &R3, &RT3, &DD3), "target");

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
        debug::err(format!("HTTP server could not connect on {}", address));
    }
}


pub fn target_all_launch(supra: String, infra: String, R: &DataRegistry, RT: &TextRegistry, DICT: &DictRegistry) {
    for i in BROKERS {
        target_broker(i.to_lowercase(), supra.to_string(), infra.to_string(), R, RT, DICT);
    }
}

pub fn target_broker(broker: String, supra: String, infra: String, R: &DataRegistry, RT: &TextRegistry, DICT: &DictRegistry) {
    println!("target {}", broker);
    if let Ok(D) = DICT.read() {
        let DD: &Dictionary = &*D;
        let nameopt = DD.infrasupraToRawName(&broker, &infra, &supra);
        if nameopt.is_some() {
            let pair = nameopt.unwrap();
            let R2 = R.clone();
            target_broker_launch(broker.to_string(), infra.to_string(), supra.to_string(), pair.to_string(), R2, RT, DICT)
        } else {
            debug::warn(format!("target_broker no rawname for {}{}{}", broker, infra, supra))
        }
    } else {
        debug::err(format!("target_broker cannot open dict"))
    }
}

pub fn route_target_all(req: &mut Request, R: &DataRegistry, RT: &TextRegistry, DICT: &DictRegistry) -> IronResult<Response> {
    let ref supra: &str = req.extensions.get::<Router>().unwrap().find("supra").unwrap_or("/");
    let ref infra: &str = req.extensions.get::<Router>().unwrap().find("infra").unwrap_or("/");

    target_all_launch(supra.to_string(), infra.to_string(), R, RT, DICT);

    let mut res = Response::with((status::Ok, "OK"));
    res.headers.set(iron::headers::AccessControlAllowOrigin::Any);
    Ok(res)
}

pub fn target(req: &mut Request, R: &DataRegistry, RT: &TextRegistry, DICT: &DictRegistry) -> IronResult<Response> {
    let ref broker: &str = req.extensions.get::<Router>().unwrap().find("broker").unwrap_or("/");
    let ref supra: &str = req.extensions.get::<Router>().unwrap().find("supra").unwrap_or("/");
    let ref infra: &str = req.extensions.get::<Router>().unwrap().find("infra").unwrap_or("/");

    target_broker(broker.to_string(), infra.to_string(), supra.to_string(), R, RT, DICT);

    let mut res = Response::with((status::Ok, "OK"));
    res.headers.set(iron::headers::AccessControlAllowOrigin::Any);
    Ok(res)
}

pub fn target_broker_launch(broker: String, infra: String, supra: String, pair: String, R: DataRegistry, RT: &TextRegistry, DICT: &DictRegistry) {
    let RT2 = RT.clone();
    let DICT2 = DICT.clone();
    println!("target broker {}", broker);
    println!("target thread {}", broker);
    match broker.as_ref() {
        "binance" => {
            let c = thread::spawn(move || {
                Universal::listen_ws_depth(TASK::WS_DEPTH, BROKER::BINANCE, pair.to_string().to_lowercase(), &R.clone());
            });
        }
        "hitbtc" => {
            let c = thread::spawn(move || {
                Universal::listen_ws_depth(TASK::WS_DEPTH, BROKER::HITBTC, pair.to_string(), &R.clone());
            });
        }

        "bitfinex" => {
            let c = thread::spawn(move || {
                Universal::listen_ws_depth(TASK::WS_DEPTH, BROKER::BITFINEX, pair.to_string(), &R.clone());
            });
        }

        "kraken" => {
            let c = thread::spawn(move || {
                let mut sched = job_scheduler::JobScheduler::new();
                sched.add(job_scheduler::Job::new("1/2 * * * * *".parse().unwrap(), || {
                    fetch::fetch_and_write_depth(BROKER::KRAKEN, supra.to_string(), infra.to_string(), &R, &RT2, &DICT2);
                }
                ));
                thread::sleep(std::time::Duration::from_millis(500));
                loop {
                    sched.tick();
                    thread::sleep(std::time::Duration::from_millis(500));
                }
            });
        }

        "kucoin" => {
            let c = thread::spawn(move || {
                let mut sched = job_scheduler::JobScheduler::new();
                sched.add(job_scheduler::Job::new("1/2 * * * * *".parse().unwrap(), || {
                    fetch::fetch_and_write_depth(BROKER::KUCOIN, supra.to_string(), infra.to_string(), &R, &RT2, &DICT2);
                }
                ));
                thread::sleep(std::time::Duration::from_millis(500));
                loop {
                    sched.tick();
                    thread::sleep(std::time::Duration::from_millis(500));
                }
            });
        }

        _ => {
            debug::err(format!("target_broker_launch UNKNWON BROKER {}", broker));
        }
    }
    //c.join();
}

pub fn get_depth(req: &mut Request) -> IronResult<Response> {
    let ref broker: &str = req.extensions.get::<Router>().unwrap().find("broker").unwrap_or("/");
    let ref pair: &str = req.extensions.get::<Router>().unwrap().find("pair").unwrap_or("/");
    let e = getEnum(broker.to_string()).unwrap();
    let text = "".to_string(); //Universal::fetch_depth(e, &pair.to_string());
    let mut res = Response::with((status::Ok, text));
    res.headers.set(iron::headers::AccessControlAllowOrigin::Any);
    Ok(res)
}

pub fn get_bidask(req: &mut Request, RT: &TextRegistry) -> IronResult<Response> {
    let ref broker: &str = req.extensions.get::<Router>().unwrap().find("broker").unwrap_or("/");
    let key: String = broker.to_string();
    let mut val: String = "".to_string();
    let key = broker.to_string();
    let RB = RT.get(&key).unwrap();
    if let Ok(st) = RB.read() {
        val = st.to_string();
    } else {
        println!("Cannot lock arc {}", broker)
    }
    let mut res = Response::with((status::Ok, val));
    res.headers.set(iron::headers::AccessControlAllowOrigin::Any);
    Ok(res)
}

pub fn get_pair(req: &mut Request, R: &DataRegistry) -> IronResult<Response> {
    let ref pair: &str = req.extensions.get::<Router>().unwrap().find("pair").unwrap_or("/");
    let mut res: String = "{".to_string();
    let mut first = true;
    for i in 0..BROKERS.len() {
        let broker: &str = BROKERS[i];
        let RB = R.get(broker).unwrap();
        if let Ok(hm) = RB.read() {
            let Q: Option<&RegistryData> = hm.get(&pair.to_string().to_uppercase());
            match Q {
                Some(qq) => {
                    let sti = hmi_to_text(pair.to_string(), qq, false);
                    if first {
                        res = format!("{}\"{}\":{}", res, broker, sti);
                    } else {
                        res = format!("{},\"{}\":{}", res, broker, sti);
                    }
                    first = false;
                }
                None => {
                    //println!("nothing for this pair {} {}",broker, pair);
                }
            }
        } else { println!("err cannot read rwlock {}", pair) }
    }
    res = format!("{}}}", res);
    let mut res = Response::with((status::Ok, res));
    res.headers.set(iron::headers::AccessControlAllowOrigin::Any);
    Ok(res)
}

pub fn get_arbitrage(req: &mut Request, R: &DataRegistry, DICT: &DictRegistry) -> IronResult<Response> {
    let ref infra: &str = req.extensions.get::<Router>().unwrap().find("infra").unwrap_or("/");
    let ref supra: &str = req.extensions.get::<Router>().unwrap().find("supra").unwrap_or("/");
    let ref budget: &str = req.extensions.get::<Router>().unwrap().find("budget").unwrap_or("/");

    let mut res: String = "{".to_string();
    let mut first = true;
    for i in 0..BROKERS.len() {
        let broker: &str = BROKERS[i];

        let RB = R.get(broker).unwrap();
        if let Ok(D) = DICT.read() {
            let DD: &Dictionary = &*D;
            let nameopt = DD.infrasupraToRawName(broker, infra, supra);
            if nameopt.is_some() {
                let pair = nameopt.unwrap();
                if let Ok(hm) = RB.read() {
                    let Q: Option<&RegistryData> = hm.get(&pair.to_string().to_uppercase());
                    match Q {
                        Some(data) => {
                            let sti = hmi_to_text(pair.to_string(), data, false);
                            if first {
                                res = format!("{}\"{}\":{}", res, broker, sti);
                            } else {
                                res = format!("{},\"{}\":{}", res, broker, sti);
                            }
                            first = false;
                        }
                        None => { println!("nothing for this pair {} {}", broker, pair); }
                    }
                } else { println!("err cannot open option bidask {}", pair) }
            } else { println!("no match {} {} {} ", broker, infra, supra) }
        }
    }


    res = format!("{}}}", res);
    let arbi = recap(budget.parse::<f64>().unwrap(), infra.to_string(), supra.to_string(), &R, &DICT);
    ;
    let fina = format!("{{\"arbitrage\":{},\"market\":{}}}", arbi, res);
    let mut res = Response::with((status::Ok, fina));
    res.headers.set(iron::headers::AccessControlAllowOrigin::Any);
    Ok(res)
}

pub fn get_arbitrage_approx(req: &mut Request, R: &DataRegistry, DICT: &DictRegistry) -> IronResult<Response> {
    let arbi = approx( &R, &DICT);

    let fina = format!("{{\"arbitrage\":{}}}", arbi);
    let mut res = Response::with((status::Ok, fina));
    res.headers.set(iron::headers::AccessControlAllowOrigin::Any);
    Ok(res)
}

pub fn handler_simple(_req: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, "Up")))
}

pub fn handler_favicon(_req: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, "Favicon")))
}

//stringifies the Data Hashmap
pub fn hm_to_text(hm: &HashMap<String, RegistryData>) -> String {
    //println!("hm");
    let mut st = "{".to_string();
    let mut first = true;
    for (symbol, data) in hm.iter() {
        //println!("{} {}",bids,asks);
        let sti = hmi_to_text(symbol.to_string(), data, true);
        if first {
            st = format!("{}{}", st, sti);
        } else {
            st = format!("{},{}", st, sti);
        }
        first = false;
    }
    //println!("hmd");
    //println!("hmd {}",st);
    format!("{}}}", st)
}

//stringifies the Data Hashmap
pub fn hmi_to_text(symbol: String, data: &RegistryData, showSymbol: bool) -> String {
    let bid: String;
    let ask: String;
    let last: String;
    match data.bid {
        Some(ref b) => { bid = format!("{}", b.to_string()); }
        None => { bid = "null".to_string(); }
    }
    match data.ask {
        Some(ref b) => { ask = format!("{}", b.to_string()); }
        None => { ask = "null".to_string(); }
    }
    match data.last {
        Some(ref b) => { last = format!("{}", b.to_string()); }
        None => { last = "null".to_string(); }
    }

    /*  for (price, size) in data.get_bids().iter() {
          println!("bids noorder{}{}",price,size);
      }*/


    let bids = Orderbook_to_string(data.get_bids(), true, true);
    let asks = Orderbook_to_string(data.get_asks(), true, false);
    if showSymbol {
        format!("\"{}\":{{\"http\":{{\"bid\":{},\"ask\":{},\"last\":{}}},\"ws\":{{\"bids\":{},\"asks\":{}}}}}", symbol, bid, ask, last, bids, asks)
    } else {
        format!("{{\"http\":{{\"bid\":{},\"ask\":{},\"last\":{}}},\"ws\":{{\"bids\":{},\"asks\":{}}}}}", bid, ask, last, bids, asks)
    }
}

pub fn orderbook_to_ordered(orderbook: &OrderbookSide, sort_order: bool) -> Vec<(f64, String, f64)> {//(price,price_str,qty)
    let mut v: Vec<(f64, String, f64)> = Vec::new();
    for (price, size) in orderbook.iter() {
        v.push((price.parse::<f64>().unwrap(), price.to_string(), *size))
    }
    if sort_order {
        v.sort_by(|&(b, _, _), &(a, _, _)| a.partial_cmp(&b).unwrap());
    } else {
        v.sort_by(|&(a, _, _), &(b, _, _)| a.partial_cmp(&b).unwrap());
    }
    v
}

fn Orderbook_to_string(orderbook: &OrderbookSide, order: bool, sort_order: bool) -> String {
    if !order {
        format!("{:?}", orderbook)
    } else {
        //put prices in vec to sort
        let mut v: Vec<(f64, String)> = Vec::new();
        for (price, size) in orderbook.iter() {
            v.push((price.parse::<f64>().unwrap(), price.to_string()))
        }
        if sort_order {
            v.sort_by(|&(b, _), &(a, _)| a.partial_cmp(&b).unwrap());
        } else {
            v.sort_by(|&(a, _), &(b, _)| a.partial_cmp(&b).unwrap());
        }
        //output in order
        let mut result = "[".to_string();
        let mut st = "";
        for price in v.iter() {
            result = format!("{}{}[{},{}]", result, st, price.0, orderbook.get(&price.1.to_string()).unwrap());
            st = ",";
        }
        result = format!("{}]", result);
        result
    }
}

