use std::thread;
use std::collections::HashMap;
use iron::{Request, Response, Chain, IronResult, Iron};
use Universal;
use Universal::{Data, Universal_DepthData, RegistryData};
use iron::status;

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
use definitions;

pub fn target(req: &mut Request, R: &DataRegistry) -> IronResult<Response> {
    let ref broker: &str = req.extensions.get::<Router>().unwrap().find("broker").unwrap_or("/");
    let ref pair: &str = req.extensions.get::<Router>().unwrap().find("pair").unwrap_or("/");

    //let c=thread::spawn(move || {
    if broker.to_string() == "binance" {
        Universal::listen_ws_depth(TASK::WS_DEPTH, BROKER::BINANCE, pair.to_string().to_lowercase(), &R.clone());
    }
    //});
    //c.join();

    let mut res = Response::with((status::Ok, "OK"));
    res.headers.set(iron::headers::AccessControlAllowOrigin::Any);
    Ok(res)
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
    if let Ok(mut st) = RB.read() {

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
                None => { println!("nothing for this pair {} {}",broker, pair);}
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
            let DD: &definitions::Dictionary = &*D;
            let nameopt = DD.infrasupraToRawName(broker, infra, supra);
            if nameopt.is_some() {
                let pair=nameopt.unwrap();
                if let Ok(hm) = RB.read() {
                    println!("len {} {}",broker, hm.len());
                    for key in hm.keys() {
                        println!("{}", key);
                    }
                    let Q: Option<&RegistryData> = hm.get(&pair.to_string());
                    match Q {
                        Some(data) => {
                            println!("{}{}{:?}",broker,pair,data.bids);
                            let sti = hmi_to_text(pair.to_string(), data, false);
                            if first {
                                res = format!("{}\"{}\":{}", res, broker, sti);
                            } else {
                                res = format!("{},\"{}\":{}", res, broker, sti);
                            }
                            first = false;
                        }
                        None => { println!("nothing for this pair {} {}",broker,pair);}
                    }
                } else { println!("err cannot open option bidask {}", pair) }
            } else {  println!("no match {} {} {} ",broker,infra,supra) }
        }
    }
    res = format!("{}}}", res);
    let mut res = Response::with((status::Ok, res));
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
    let bids: String;
    let asks: String;
    match data.bid {
        Some(ref b) => { bid = format!("{}", b.to_string()); }
        None => { bid = "null".to_string(); }
    }
    match data.ask {
        Some(ref b) => { ask = format!("{}", b.to_string()); }
        None => { ask = "null".to_string(); }
    }
    bids = format!("{:?}", data.bids);
    asks = format!("{:?}", data.asks);
    match data.last {
        Some(ref b) => { last = format!("{}", b.to_string()); }
        None => { last = "null".to_string(); }
    }

    println!("{} {}",bids,asks);
    if showSymbol {
        format!("\"{}\":{{\"bid\":{},\"ask\":{},\"last\":{},\"bids\":{},\"asks\":{}}}", symbol, bid, ask, last, bids, asks)
    } else {
        format!("{{\"bid\":{},\"ask\":{},\"last\":{},\"bids\":{},\"asks\":{}}}", bid, ask, last, bids, asks)
    }
}
