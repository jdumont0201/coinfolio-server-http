use iron::{Request,Response,Chain,IronResult,Iron};
use Universal;
use iron::status;
use iron;
use router::{Router, NoRoute};
use BidaskTextRegistry;
use BidaskRegistry;

pub fn get_depth(req: &mut Request) -> IronResult<Response> {
    let ref broker: &str = req.extensions.get::<Router>().unwrap().find("broker").unwrap_or("/");
    let ref pair: &str = req.extensions.get::<Router>().unwrap().find("pair").unwrap_or("/");
    let text= Universal::fetch_depth(&broker.to_string(), &pair.to_string());


    let mut res = Response::with((status::Ok, text));
    res.headers.set(iron::headers::AccessControlAllowOrigin::Any);
    Ok(res)
}
pub fn get_bidask(req: &mut Request, ticker: &BidaskTextRegistry) -> IronResult<Response> {
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

pub fn handler_simple(_req: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, "Up")))
}

pub fn handler_favicon(_req: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, "Favicon")))
}