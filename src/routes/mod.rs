use std::thread;
use OrderbookSide;
use std::iter::FromIterator;
use std::collections::HashMap;
use iron::{Request, Response, Chain, IronResult, Iron};
use Universal;
use Universal::{Data, Universal_Orderbook, RegistryData};
use iron::status;
use arbitrage::recap;
use iron;

use std::cell::RefCell;
use std::sync::RwLock;
use router::{Router, NoRoute};
use BidaskTextRegistry;
use BidaskReadOnlyRegistry;
use BidaskRegistry;
use DataRegistry;
use TextRegistry;
use DictRegistry;
use Brokers::{BROKER, getEnum, TASK, BROKERS};
use dictionary::Dictionary;
use middlewares;

pub fn start_http_server(RT: &TextRegistry,R:&DataRegistry,DICT:&DictRegistry) {
    println!("Coinamics Server HTTP");
    //create routes
    let mut router = Router::new();
    router.get("/", handler_simple, "index");
    router.get("/favicon.ico", handler_favicon, "favicon");
    let RT2 = RT.clone();
    router.get("/exchange/:broker/price", move |request: &mut Request| get_bidask(request, &RT2), "ticker");
    let R2 = R.clone();
    router.get("/pair/:pair", move |request: &mut Request| get_pair(request, &R2), "pair");
    let R2b = R.clone();

    let DD2=DICT.clone();
    router.get("/task/arbitrage/supra/:supra/infra/:infra", move |request: &mut Request| get_infrasupra(request, &R2b,&DD2), "infrasupra");
    router.get("/exchange/:broker/task/depth/symbol/:pair", move |request: &mut Request| get_depth(request), "depth");
    let R3=R.clone();
    let DD3=DICT.clone();
    router.get("/task/target/exchange/:broker/supra/:supra/infra/:infra", move |request: &mut Request| target(request,&R3,&DD3), "target");

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




pub fn target(req: &mut Request, R: &DataRegistry, DICT: &DictRegistry) -> IronResult<Response> {
    let ref broker: &str = req.extensions.get::<Router>().unwrap().find("broker").unwrap_or("/");
    let ref supra: &str = req.extensions.get::<Router>().unwrap().find("supra").unwrap_or("/");
    let ref infra: &str = req.extensions.get::<Router>().unwrap().find("infra").unwrap_or("/");


    if let Ok(D) = DICT.read() {
        let DD: &Dictionary = &*D;
        let nameopt = DD.infrasupraToRawName(broker, infra, supra);
        if nameopt.is_some() {
            let pair = nameopt.unwrap();
            let R2 = R.clone();
            startTarget(broker.to_string(), pair.to_string(), R2)
        }
    }

    let mut res = Response::with((status::Ok, "OK"));
    res.headers.set(iron::headers::AccessControlAllowOrigin::Any);
    Ok(res)
}

pub fn startTarget(broker: String, pair: String, R: DataRegistry) {
    let c = thread::spawn(move || {
        match broker.as_ref() {
            "binance" => {
                Universal::listen_ws_depth(TASK::WS_DEPTH, BROKER::BINANCE, pair.to_string().to_lowercase(), &R.clone());
            }
            "hitbtc" => {
                Universal::listen_ws_depth(TASK::WS_DEPTH, BROKER::HITBTC, pair.to_string().to_lowercase(), &R.clone());
            },_=>{

            }
        }
    });
    c.join();
}

pub fn get_depth(req: &mut Request) -> IronResult<Response> {
    let ref broker: &str = req.extensions.get::<Router>().unwrap().find("broker").unwrap_or("/");
    let ref pair: &str = req.extensions.get::<Router>().unwrap().find("pair").unwrap_or("/");

    let e = getEnum(broker.to_string()).unwrap();
    let text = Universal::fetch_depth(e, &pair.to_string());


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
            let Q: Option<&RegistryData> = hm.get(&pair.to_string());
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
                None => { //println!("nothing for this pair {} {}",broker, pair);
                }
            }
        } else { println!("err cannot read rwlock {}", pair) }
    }
    res = format!("{}}}", res);
    let mut res = Response::with((status::Ok, res));
    res.headers.set(iron::headers::AccessControlAllowOrigin::Any);
    Ok(res)
}

pub fn get_infrasupra(req: &mut Request, R: &DataRegistry, DICT: &DictRegistry) -> IronResult<Response> {
    let ref infra: &str = req.extensions.get::<Router>().unwrap().find("infra").unwrap_or("/");
    let ref supra: &str = req.extensions.get::<Router>().unwrap().find("supra").unwrap_or("/");

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
                    let Q: Option<&RegistryData> = hm.get(&pair.to_string());
                    match Q {
                        Some(data) => {
                            for (price, size) in data.get_bids().iter() {
                                println!("req{}{}", price, size);
                            }
                            //println!("{}{}{:?}",broker,pair,data.bids);
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
    let arbi=recap(1000.,infra.to_string(),supra.to_string(),&R,&DICT);;
    let fina=format!("{{\"arbitrage\":{},\"market\":{}}}",arbi,res);
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
pub fn orderbook_to_ordered(orderbook:&OrderbookSide,sort_order:bool)-> Vec<(f64,String,f64)>{
    let mut v: Vec<(f64, String,f64)> = Vec::new();
    for (price, size) in orderbook.iter() {
        v.push((price.parse::<f64>().unwrap(), price.to_string(),*size))
    }
    if sort_order {
        v.sort_by(|&(b, _,_), &(a, _,_)| a.partial_cmp(&b).unwrap());
    } else {
        v.sort_by(|&(a, _,_), &(b, _,_)| a.partial_cmp(&b).unwrap());
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
//            println!("tostr {}{}",price.0,orderbook.get(&price.1.to_string()).unwrap());
            result = format!("{}{}[{},{}]", result, st, price.0, orderbook.get(&price.1.to_string()).unwrap());
            st = ",";
        }
        result = format!("{}]", result);
        result
    }
}

