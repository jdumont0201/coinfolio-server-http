use std;
use Data;
use std::collections::HashMap;
use OrderbookSide;
use serde_json;
use Brokers::BROKER;
use Universal::Universal_Orderbook;
use Universal::Universal_Orderbook_in;
use RefreshData;
use DataRegistry;
use ws::{listen, connect, Handshake, Handler, Sender, Result as wsResult, Message, CloseCode};

static NAME: &str = "hitbtc";
pub static URL_HTTP_BIDASK: &str = "https://api.hitbtc.com/api/2/public/ticker";
pub static URL_WS_DEPTH: &str = "wss://api.hitbtc.com/api/2/ws";

#[derive(Serialize, Deserialize)]
pub struct Bidask {
    ask: Option<String>,
    bid: Option<String>,
    last: Option<String>,
    open: Option<String>,
    low: Option<String>,
    high: Option<String>,
    volume: Option<String>,
    volumeQuote: Option<String>,
    timestamp: String,
    symbol: String,
}

//WS DEPTH
#[derive(Serialize, Deserialize)]
pub struct WSDepth {
    pub jsonrpc: String,
    pub method: Option<String>,
    pub params: WSDepth_in,
    pub result: Option<bool>,
}

#[derive(Serialize, Deserialize)]
pub struct WSDepth_in {
    pub ask: Vec<WSDepth_in_in>,
    pub bid: Vec<WSDepth_in_in>,
}

#[derive(Serialize, Clone, Deserialize)]
pub struct WSDepth_in_in {
    pub price: String,
    pub size: String,
}


//WS DEPTH CLIENT
pub struct WSDepthClient {
    pub out: Sender,
    pub broker: BROKER,
    pub registry: DataRegistry,
    pub symbol: String,
}

impl Handler for WSDepthClient {
    fn on_open(&mut self, _: Handshake) -> wsResult<()> {
        let json = format!("{{ \"method\": \"subscribeOrderbook\",\"params\": {{\"symbol\": \"{}\"}},\"id\": 123 }}", self.symbol);
        println!("Open ws {} {} ", self.broker, json);
        self.out.send(json)
    }
    fn on_message(&mut self, msg: Message) -> wsResult<()> {
        let msg2 = msg.to_string();
        let msg3 = str::replace(&msg2, ",[]", "");

        let parsedMsg: Result<WSDepth, serde_json::Error> = serde_json::from_str(&msg3);
        match parsedMsg {
            Ok(parsedMsg_) => {
                if !parsedMsg_.result.is_some() {
                    let bid = parsedMsg_.params.bid.clone();
                    let ask = parsedMsg_.params.ask.clone();

                    let mut orderbook_bid: OrderbookSide = HashMap::new();

                    let mut i = 0;
                    for item in bid.iter() {
                            orderbook_bid.insert(item.price.clone(), item.size.clone().parse::<f64>().unwrap());
                    }
                    let mut orderbook_ask: OrderbookSide = HashMap::new();
                    let mut i=0;
                    for item in ask.iter() {
                           orderbook_ask.insert(item.price.clone(), item.size.clone().parse::<f64>().unwrap());

                    }
                    let D = Universal_Orderbook { bids: orderbook_bid, asks: orderbook_ask };

                    if parsedMsg_.method.is_some() {
                        match parsedMsg_.method.unwrap().as_ref() {
                            "snapshotOrderbook" => {
                                let mut i=0;
                                for item in bid.iter() {

                                    i=i+1

                                }
                                RefreshData::snapshot_depth(self.broker, &self.registry, self.symbol.to_string(), D);
                            }
                            "updateOrderbook" => {
//                                RefreshData::update_depth(self.broker, &self.registry, self.symbol.to_string(), D);
                            }
                            _ => {}
                        }
                    }
                }
            }
            Err(err) => { println!("cannot unmarshal {} ws depth {}", NAME, err) }
        }
        Ok(())
    }
    fn on_close(&mut self, code: CloseCode, reason: &str) {
        match code {
            CloseCode::Normal => println!("The client is done with the connection."),
            CloseCode::Away => { println!("The client is leaving the site. Update room count"); }
            CloseCode::Abnormal => println!("Closing handshake failed! Unable to obtain closing status from client."),
            CloseCode::Protocol => println!("protocol"),
            CloseCode::Unsupported => println!("Unsupported"),
            CloseCode::Status => { println!("Status"); }
            CloseCode::Abnormal => println!("Abnormal"),
            CloseCode::Invalid => println!("Invalid"),
            CloseCode::Protocol => println!("protocol"),
            CloseCode::Policy => println!("Policy"),
            CloseCode::Size => println!("Size"),
            CloseCode::Extension => println!("Extension"),
            CloseCode::Protocol => println!("protocol"),
            CloseCode::Restart => println!("Restart"),
            CloseCode::Again => println!("Again"),

            _ => println!("CLOSE The client encountered an error: {}", reason),
        }
    }
}


pub fn parse_bidask(text: String) -> HashMap<String, Data> {
    let mut r = HashMap::new();

    let bs: Result<Vec<Bidask>, serde_json::Error> = serde_json::from_str(&text);
    match bs {
        Ok(bs_) => {
            for row in bs_ {
                r.insert(row.symbol, Data { bid: row.bid, ask: row.ask, last: row.last });
            }
        }
        Err(err) => {
            println!(" !!! cannot unmarshall bidask {} {:?}", NAME, err)
        }
    }
    r
}