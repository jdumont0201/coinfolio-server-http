use std::thread;
use std::collections::HashMap;
use iron::{Request, Response, Chain, IronResult, Iron};
use Universal;
use Universal::{Data, DepthData, RegistryData};
use iron::status;
use RefreshData::hmi_to_text;
use iron;
use router::{Router, NoRoute};
use BidaskTextRegistry;
use BidaskReadOnlyRegistry;
use BidaskRegistry;
use DataRegistry;
use TextRegistry;
use Brokers::{BROKER, getEnum, TASK, BROKERS};

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

    println!("getp");
    for i in 0..BROKERS.len() {
        let key: &str = BROKERS[i];
        let RB = R.get(key).unwrap();
        if let Ok(hm) = RB.read() {
            //if let Some(ref hm) = *opt { //open option
            println!("getp open");
            println!("getp {}", key);
            println!("getp {} ok", key);
            let Q: Option<&RegistryData> = hm.get(&pair.to_string());
            match Q {
                Some(qq) => {
                    println!("getp {} ok w", key);
                    let sti = hmi_to_text(pair.to_string(), qq, false);
                    if first {
                        res = format!("{}\"{}\":{}", res, key, sti);
                    } else {
                        res = format!("{},\"{}\":{}", res, key, sti);
                    }
                    first = false;
                }
                None => {}
            }
            //let sti = hmi_to_text(pair.to_string(), vv);
            //val = format!("{},{}:{}", val, key, sti);
        } else { println!("err cannot open option bidask {}", pair) }
        //} else { println!("err cannot lock arcmutex bidask {}", pair) }
    }
    res = format!("{}}}", res);
    println!("getp ok d");
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