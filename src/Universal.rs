
    use serde_json;
    use reqwest;
    use std::collections::HashMap;

    pub struct Data {
        pub bid: Option<String>,
        pub ask: Option<String>,
        pub last: Option<String>,
    }
    pub struct DepthData {
        pub bids: Vec<Vec<f64>>,
        pub asks: Vec<Vec<f64>>,

    }

    #[derive(Serialize, Deserialize)]
    struct binance_depth {
        lastUpdateId: String,
        bids: Vec<Vec<f64>>,
        asks: Vec<Vec<f64>>
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
    struct binance_price {
        symbol: String,
        price: String,
    }

    #[derive(Serialize, Deserialize)]
    struct kucoin_bidask {
        success: bool,
        code: String,
        msg: String,
        timestamp: u64,
        data: Vec<kucoin_bidask_in>,
    }

    #[derive(Serialize, Deserialize)]
    struct kucoin_bidask_in {
        coinType: String,
        trading: bool,
        symbol: String,
        lastDealPrice: Option<f64>,
        buy: Option<f64>,
        sell: Option<f64>,
        change: Option<f64>,
        coinTypePair: Option<String>,
        sort: Option<u64>,
        feeRate: Option<f64>,
        volValue: Option<f64>,
        high: Option<f64>,
        datetime: Option<u64>,
        vol: Option<f64>,
        low: Option<f64>,
        changeRate: Option<f64>,
    }


    #[derive(Serialize, Deserialize)]
    struct hitbtc_bidask {
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

    #[derive(Serialize, Deserialize)]
    struct cryptopia_bidask {
        Success:bool,
        Message:Option<String>,
        Error:Option<String>,
        Data:Vec<cryptopia_bidask_in>
    }

    #[derive(Serialize, Deserialize)]
    struct cryptopia_bidask_in {
        TradePairId: u64,
        Label: String,
        AskPrice: f64,
        BidPrice: f64,
        Low: f64,
        High: f64,
        Volume: f64,
        LastPrice: f64,
        BuyVolume: f64,
        SellVolume: f64,
        Change: f64,
        Open: f64,
        Close:f64,
        BaseVolume: f64,
        BuyBaseVolume: f64,
        SellBaseVolume: f64,
    }

    #[derive(Serialize, Deserialize)]
    struct kraken_bidask {
        error: Vec<String>,
        result: HashMap<String, kraken_bidask_in>,
    }

    #[derive(Serialize, Deserialize)]
    struct kraken_bidask_in {
        a: Option<Vec<String>>,
        b: Option<Vec<String>>,
        c: Option<Vec<String>>,
        v: Option<Vec<String>>,
        p: Option<Vec<String>>,
        t: Option<Vec<u64>>,
        l: Option<Vec<String>>,
        h: Option<Vec<String>>,
        o: Option<String>,
    }


    fn getGeneric_hashmap(task: String, broker: String, text: String) -> HashMap<String, Data> {
        let mut r = HashMap::new();
        if task == "bidask" {
            if broker == "binance" {
                let bs: Result<Vec<binance_bidask>,serde_json::Error>  = serde_json::from_str(&text);
                if let Ok(bs_) = bs {
                    for row in bs_ {
                        r.insert(row.symbol, Data { bid: Some(row.bidPrice), ask: Some(row.askPrice), /* bidq: Some(row.bidQty), askq: Some(row.askQty),*/ last: None });
                    }
                }else{
                    println!(" !!! cannot unmarshall {} {}",task,broker)
                }
            } else if broker == "hitbtc" {
                //println!("format {} {} ",broker,text);
                let bs: Result<Vec<hitbtc_bidask>,serde_json::Error>  = serde_json::from_str(&text);
                if let Ok(bs_) = bs {
                    for row in bs_   {
                        r.insert(row.symbol, Data { bid: row.bid, ask: row.ask, last: row.last });
                    }
                }else{
                    println!(" !!! cannot unmarshall {} {}",task,broker)
                }
            }  else if broker == "cryptopia" {

                let bs: Result<cryptopia_bidask,serde_json::Error>  = serde_json::from_str(&text);
                match bs {
                    Ok(bs_) => {
                        for row in bs_.Data {
                            let symb = str::replace(&row.Label, "/", "");
                            r.insert(symb, Data { bid: Some(row.BidPrice.to_string()), ask: Some(row.AskPrice.to_string()), last: Some(row.LastPrice.to_string()) });
                        }
                    },
                    Err(err) => {
                        println!(" !!! cannot unmarshall {} {} {:?}", task, broker, err)
                    }
                }
            } else if broker == "kucoin" {
                let bs: Result<kucoin_bidask,serde_json::Error> = serde_json::from_str(&text);
                match bs {
                    Ok(bs_) => {
                        for row in bs_.data {
                            //println!("format {} {} ",broker,text);
                            let symb = str::replace(&row.symbol, "-", "");
                            let mut b;                            if let Some(bb) = row.buy { b = Some(bb.to_string()) } else { b = None }
                            let mut la;                            if let Some(la_) = row.buy { la = Some(la_.to_string()) } else { la = None }
                            let mut se;                            if let Some(se_) = row.sell { se = Some(se_.to_string()) } else { se = None }

                            r.insert(symb, Data { bid: b, ask: se, last: la });
                        }
                    },
                    Err(err) => {
                        println!(" !!! cannot unmarshall {} {} {:?}", task, broker,err)
                    }
                }

            } else if broker == "kraken" {

                let bs: Result<kraken_bidask,serde_json::Error> = serde_json::from_str(&text);

                match bs {
                    Ok(bs_) => {
                        for (symbol,row) in bs_.result.iter() {
                            let mut b; match row.b {Some(ref b_)=>{ b=Some(b_[0].to_string())}  ,None=>{b=Some("".to_string())} }
                            let mut a; match row.a {Some(ref a_)=>{ a=Some(a_[0].to_string())}  ,None=>{a=Some("".to_string())} }
                            let mut c; match row.c {Some(ref c_)=>{ c=Some(c_[0].to_string())}  ,None=>{c=Some("".to_string())} }
                            r.insert(symbol.to_string(), Data { bid: b, ask: a, last: c });
                        }
                    },
                    Err(err) => {
                        println!(" !!! cannot unmarshall {} {} {:?}", task, broker,err)
                    }
                }
            }
        } else if task == "price" {
            if broker == "binance" {

                let bs:Result<Vec<binance_price>,serde_json::Error>= serde_json::from_str(&text);
                if let Ok(bs_)= bs {
                    for row in bs_ {
                        r.insert(row.symbol, Data { bid: None, ask: None, last: Some(row.price) });
                    }
                }else{
                    println!(" !!! cannot unmarshall {} {}",task,broker)
                }
            }
        }
        r
    }
    fn getGeneric_depth_hashmap(task: String, broker: String, text: String) -> String {
        let mut r : String="".to_string();
        if task == "depth" {
            if broker == "binance" {
                let text2 = str::replace(&text, ",[]", "");

                r=text2;

            }
        }
        r
    }
    pub fn fetch_bidask(broker: &String) -> HashMap<String, Data> {
        println!("fetch bidask {}", broker);
        let url = get_url("bidask".to_string(), broker);
        let mut result: HashMap<String, Data>;
        if let Ok(mut res) = reqwest::get(&url) {
            let getres = match res.text() {
                Ok(val) => {
                    let v = getGeneric_hashmap("bidask".to_string(), broker.to_string(), val);
                    println!("{} {}", broker, broker);
                    result = v;
                }
                Err(err) => {
                    println!("[GET_BIDASK] err");
                    result = HashMap::new();
                }
            };
        } else {
            result = HashMap::new();
        }
        result
    }


    pub fn fetch_depth(broker: &String,pair:&String) -> String {
        println!("fetch string {}", broker);
        let url = format!("{}{}",get_url("depth".to_string(), broker),pair);
        let mut result: String;
        if let Ok(mut res) = reqwest::get(&url) {
            let getres = match res.text() {
                Ok(val) => {
                    let v = getGeneric_depth_hashmap("depth".to_string(), broker.to_string(), val);
                    println!("{} {}", broker, broker);
                    result = v;
                }
                Err(err) => {
                    println!("[GET_DEPTH] err");
                    result = "".to_string()
                }
            };
        } else {
            result = "".to_string()
        }
        result
    }

    pub fn fetch_price(broker: &String) -> HashMap<String, Data> {
        //println!("fetch price {}",broker);
        let url = get_url("price".to_string(), &broker);

        let mut result: HashMap<String, Data>;
        if let Ok(mut res) = reqwest::get(&url) {
            let getres = match res.text() {
                Ok(val) => {
                    let v = getGeneric_hashmap("price".to_string(), broker.to_string(), val);
                    result = v;
                }
                Err(err) => {
                    println!("[GET_PRICE] err");
                    result = HashMap::new();
                }
            };
        } else {
            result = HashMap::new();
        }
        result
    }

    fn get_url(task: String, broker: &String) -> String {
        let mut r = "".to_string();
        if task == "bidask" {
            if broker == "binance" {
                r = "https://api.binance.com/api/v1/ticker/bookTicker".to_string();
            } else if broker == "hitbtc" {
                r = "https://api.hitbtc.com/api/2/public/ticker".to_string();
            } else if broker == "kraken" {
                r = "https://api.kraken.com/0/public/Ticker?pair=BCHUSD,BCHXBT,DASHUSD,DASHXBT,EOSXBT,GNOXBT,USDTZUSD,XETCXXBT,XETCZUSD,XETHXXBT,XETHZUSD,XETHZUSD.d,XICNXXBT,XLTCXXBT,XLTCZUSD,XMLNXXBT,XREPXXBT,XXBTZCAD,XXBTZCAD.d,XXBTZUSD,XXBTZUSD.d,XXDGXXBT,XXLMXXBT,XXMRXXBT,XXMRZUSD,XXRPXXBT,XXRPZUSD,XZECXXBT,XZECZUSD".to_string()
            } else if broker == "kucoin" {
                r = "https://api.kucoin.com/v1/open/tick".to_string()
            } else if broker == "cryptopia" {
                r = "https://www.cryptopia.co.nz/api/GetMarkets".to_string()
            }
        } else if task == "price" {
            if broker == "binance" {
                r = "https://api.binance.com/api/v3/ticker/price".to_string();
            } else if broker == "hitbtc" {
                r = "https://api.hitbtc.com/api/2/public/ticker".to_string();
            }
        }else if task=="depth"{
            if broker=="binance" {
                r="https://api.binance.com/api/v1/depth?symbol=".to_string()
            }
        }
        r
    }
