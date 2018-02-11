use std;use Data;
use std::collections::HashMap;
use OrderbookSide;
use serde_json;
use Brokers::BROKER;
use Universal::Universal_Orderbook;
use Universal::Universal_Orderbook_in;
use RefreshData;
use DataRegistry;
use ws::{listen, connect, Handshake, Handler, Sender, Result as wsResult, Message, CloseCode};

static NAME:&str="bitfinex";
pub static URL_HTTP_BIDASK:&str="https://api.bitfinex.com/v2/tickers?symbols=tBTCUSD,tLTCUSD,tLTCBTC,tETHUSD,tETHBTC,tETCBTC,tETCUSD,tRRTUSD,tRRTBTC,tZECUSD,tZECBTC,tXMRUSD,tXMRBTC,tDSHUSD,tDSHBTC,tBTCEUR,tXRPUSD,tXRPBTC,tIOTUSD,tIOTBTC,tIOTETH,tEOSUSD,tEOSBTC,tEOSETH,tSANUSD,tSANBTC,tSANETH,tOMGUSD,tOMGBTC,tOMGETH,tBCHUSD,tBCHBTC,tBCHETH,tNEOUSD,tNEOBTC,tNEOETH,tETPUSD,tETPBTC,tETPETH,tQTMUSD,tQTMBTC,tQTMETH,tAVTUSD,tAVTBTC,tAVTETH,tEDOUSD,tEDOBTC,tEDOETH,tBTGUSD,tBTGBTC,tDATUSD,tDATBTC,tDATETH,tQSHUSD,tQSHBTC,tQSHETH,tYYWUSD,tYYWBTC,tYYWETH,tGNTUSD,tGNTBTC,tGNTETH,tSNTUSD,tSNTBTC,tSNTETH,tIOTEUR,tBATUSD,tBATBTC,tBATETH,tMNAUSD,tMNABTC,tMNAETH,tFUNUSD,tFUNBTC,tFUNETH,tZRXUSD,tZRXBTC,tZRXETH,tTNBUSD,tTNBBTC,tTNBETH,tSPKUSD,tSPKBTC,tSPKETH,tTRXUSD,tTRXBTC,tTRXETH,tRCNUSD,tRCNBTC,tRCNETH,tRLCUSD,tRLCBTC,tRLCETH,tAIDUSD,tAIDBTC,tAIDETH,tSNGUSD,tSNGBTC,tSNGETH,tREPUSD,tREPBTC,tREPETH,tELFUSD,tELFBTC,tELFETH";
pub static URL_WS_DEPTH: &str = "wss://api.bitfinex.com/ws/";
pub fn parse_bidask(text:String) -> HashMap<String,Data>{
    let mut r = HashMap::new();
    let text2b = str::replace(&text, "[[", "");
    let text2 = str::replace(&text2b, "]]", "");
    let bs: Vec<&str> = text2.split("],[").collect();
    if bs.len() > 0 {
        for row in bs.iter() {
//println!("row {:?}",row);
            let rows: Vec<&str> = row.split(",").collect();
            if rows.len() > 6 {
                let symbol = str::replace(&rows[0], "\"", "");
                let bid = rows[1];
                let bidQ = rows[2];
                let ask = rows[3];
                let askQ = rows[4];
                let last = rows[7];
//println!(" {} {} {} {}",symbol,bid,ask,last);
                r.insert(symbol.to_string(), Data { bid: Some(bid.to_string()), ask: Some(ask.to_string()), /* bidq: Some(row.bidQty), askq: Some(row.askQty),*/ last: Some(last.to_string()) });
            }
        }
    }else{
        println!(" !!! cannot read bidask  {}", NAME)
    }
    r
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
        let json = format!("{{  \"event\":\"subscribe\",   \"channel\":\"book\",   \"pair\":\"{}\", \"prec\":\"{}\",\"length\":\"{}\",\"freq\":\"{}\"}}", self.symbol,"P0",100,"F0");
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
                        if i<10 {
                            orderbook_bid.insert(item.price.clone(), item.size.clone().parse::<f64>().unwrap());
                        }
                        i=i+1;
                    }
                    let mut orderbook_ask: OrderbookSide = HashMap::new();
                    let mut i=0;
                    for item in ask.iter() {
                        //                           orderbook_ask.insert(item.price.clone(), item.size.clone().parse::<f64>().unwrap());

                    }
                    let D = Universal_Orderbook { bids: orderbook_bid, asks: orderbook_ask };

                    if parsedMsg_.method.is_some() {
                        match parsedMsg_.method.unwrap().as_ref() {
                            "snapshotOrderbook" => {
                                let mut i=0;
                                for item in bid.iter() {
                                    if i<10 {
                                        println!("bid parsmsg {} {}",item.price,item.size);

                                    }
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
