use std;
use time::Duration;
use DataRegistry;
use BROKERS;
use dictionary::Dictionary;
use RegistryData;
use DictRegistry;
use routes::orderbook_to_ordered;
use std::collections::HashMap;
use chrono::offset::{TimeZone, Utc};

pub struct Operations {
    operations: Transactions,
    name: String,
    profit: f64,
    profitpc: f64,
}

pub struct Transactions {
    transactions: HashMap<String, Transaction>,
    bestVal: f64,
    best: Option<String>,
    best_recap: String,
    name: String,
    symbol: String,
    typ: String,
    meanPrice: f64,
}

impl Transactions {
    pub fn to_json(&self) -> String {
        let mut best = "".to_string();
        match self.best {
            Some(ref b) => {
                best = b.to_string();
                let mut res = format!("{{\"best_recap\":\"{}\",\"name\":\"{}\",\"type\":\"{}\",\"symbol\":\"{}\",\"best\":\"{}\",\"brokers\":{{", self.best_recap, self.name, self.typ, self.symbol, best);
                let mut st = "";
                for (broker, t) in self.transactions.iter() {
                    res = format!("{}{}{}", res, st, t.to_json());
                    st = ","
                }
                format!("{}}}}}", res)
            }
            None => {
                let mut res = format!("{{\"recap\":\"{}\",\"name\":\"{}\",\"type\":\"{}\",\"symbol\":\"{}\",\"best\":\"{}\",\"brokers\":{{}} ", self.best_recap, self.name, self.typ, self.symbol, best);
                res
            }
        }
    }
}

struct Transaction {
    broker: String,

    budget: f64,
    commission: f64,
    commission_transfer: f64,
    tradingBudget: f64,
    orders: Vec<Level>,
    meanPrice: f64,
    remainer: f64,
    quantityTotal: f64,
    value: f64,
}

impl Transaction {
    pub fn to_json(&self) -> String {
        let mut ordersstr = "[".to_string();
        let mut st = "";
        for i in self.orders.iter() {
            ordersstr = format!("{}{}{}", ordersstr, st, i.to_json());
            st = ","
        }
        ordersstr = format!("{}]", ordersstr);

        format!("\"{}\":{{\"budget\":{}, \"commissionTrading\":{},\"commissionTransfer\":{},\"tradingBudget\":{},\"orders\":{},\"meanPrice\":{},\"quantityExchanged\":{},\"remainer\":{},\"value\":{} }}", self.broker, self.budget, self.commission,self.commission_transfer, self.tradingBudget, ordersstr, self.meanPrice, self.quantityTotal, self.remainer, self.value)
    }
}

struct Level {
    qty: f64,
    price: f64,
    value: f64,
}

impl Level {
    pub fn to_json(&self) -> String {
        format!("{{\"qty\":{},\"price\":{},\"cost\":{} }}", self.qty, self.price, self.value)
    }
}

// for each broker, reads data[PAIR] and computes cheapest ask and most expensive bid
pub fn recap(budget: f64, infra: String, supra: String, R: &DataRegistry, DICT: &DictRegistry) -> String {
    let d1 = Utc::now();

    let mut cheapestBroker: String;
    let mut cheapestAsk: Vec<(f64, f64)>;

    let TTB = optimize_single_buy(budget, infra.to_string(), supra.to_string(), R, DICT);


    match TTB.best {
        Some(ref best_) => {
            let BEST = TTB.transactions.get(best_).unwrap();
            let newbudget = BEST.quantityTotal;


            println!("OP2 SELL");
            let TTS = optimize_single_sell(newbudget, infra.to_string(), supra.to_string(), R, DICT);

            match TTS.best {
                Some(ref bests_) => {
                    let BESTS = TTS.transactions.get(bests_).unwrap();
                    let profit = BESTS.value - BEST.value;
                    let profit_pc = (profit / BEST.value) * 100.;
                    let d2 = Utc::now();
                    let duration = d2.signed_duration_since(d1);

                    let ms = duration.num_microseconds().unwrap();
                    let ms: f64 = ms as f64 / 1000.;


                    format!("{{\"computationTime_ms\":{},\"budget\":{},\"profit\":{},\"profit_pc\":{},\"recap\":[\"{}\",\"TRANSFER to {}\",\"{}\"],\"transactions\":[{{\"name\":\"first buy\",\"result\":{} }},{{\"name\":\"sell\",\"result\":{} }}]}}", ms, budget, profit, profit_pc, TTB.best_recap,TTS.best.as_ref().unwrap(),TTS.best_recap,TTB.to_json(), TTS.to_json())
                }
                None => {
                    format!("[{{\"operation\":\"first buy\",\"result\":{} }} }}]", TTB.to_json())
                }
            }
        }
        None => {
            format!("[{{\"operation\":\"first buy\",\"result\":{} }} }}]", TTB.to_json())
        }
    }
}

pub fn optimize_single_buy(budget: f64, infra: String, supra: String, R: &DataRegistry, DICT: &DictRegistry) -> Transactions {
    let mut TT = Transactions { typ: "BUY".to_string(), meanPrice: 0., best_recap: "".to_string(), name: format!("Buy {}/{}", supra, infra), symbol: format!("{}{}", supra, infra), transactions: HashMap::new(), bestVal: 1000000000., best: None };
    for i in 0..BROKERS.len() {//for each broker
        let broker: &str = BROKERS[i];
        let RB = R.get(broker).unwrap();
        if infra == "USD" {
            if let Ok(D) = DICT.read() { //open dictionary to find broker name of the pair
                let DD: &Dictionary = &*D;
                let pairopt = DD.infrasupraToRawName(broker, &infra, &supra);
                if pairopt.is_some() {//if pair exists
                    let pair = pairopt.unwrap();
                    if let Ok(hm) = RB.read() { //read registry for this pair
                        let dataOption: Option<&RegistryData> = hm.get(&pair.to_string());
                        match dataOption {
                            Some(data) => {
                                let commissionBrokerTrading = 0.001;
                                let budgetAvailable = budget;
                                let ASKS = data.get_asks();
                                let ordered: Vec<(f64, String, f64)> = orderbook_to_ordered(ASKS, false);
                                let mut budres = budgetAvailable;
                                let mut qtyres = 0.;
                                let mut quantityBought = 0.;
                                let mut meanPrice = 0.;
                                let mut invested = 0.;
                                let mut T = Transaction {
                                    broker: broker.to_string(),
                                    budget: budget,
                                    commission: 0.,
                                    commission_transfer: 0.,
                                    tradingBudget: budgetAvailable,
                                    orders: Vec::new(),
                                    meanPrice: 0.,
                                    value: 0.,
                                    quantityTotal: 0.,
                                    remainer: budget,
                                };

                                for &(ref price, ref pricestr, ref size) in ordered.iter() {
                                    if budres <= 0.000000001 { break }
                                    println!("    {} Budget {}", broker, budres);
                                    let levelQty = *size;
                                    let levelPrice = *price;
                                    let mut operationQuantityBought;
                                    let buyable = budres / levelPrice;
                                    if levelQty > buyable {
                                        operationQuantityBought = buyable;
                                    } else {
                                        operationQuantityBought = levelQty;
                                    }
                                    operationQuantityBought=operationQuantityBought ;//commission
                                    let operationCost = operationQuantityBought * levelPrice;
                                    T.orders.push(Level { qty: operationQuantityBought, price: levelPrice, value: operationCost });
                                    invested=invested+operationCost;
                                    quantityBought = quantityBought + operationQuantityBought;
                                   // meanPrice = meanPrice + levelPrice * operationQuantityBought;
                                    budres = budres - operationCost;
                                    println!("    {} buy {} {} {} {} ", broker, levelQty, levelPrice, operationCost, operationQuantityBought);
                                }
                                T.commission=invested *  commissionBrokerTrading;
                                T.commission_transfer=get_withdraw_commission(broker.to_string(),supra.to_string(),invested);
                                let residual = invested-T.commission_transfer-T.commission;
                                meanPrice = quantityBought/residual; //meanPrice / quantityBought;

                                T.quantityTotal = quantityBought;
                                T.meanPrice = meanPrice;
                                T.remainer = budres;

                                println!("com{}{}", quantityBought, commissionBrokerTrading);

                                T.value = T.remainer+ residual - T.commission_transfer;
                                T.commission = T.value * commissionBrokerTrading;
                                let mut active = T.quantityTotal > 0.;
                                if active {
                                    if T.meanPrice < TT.bestVal {
                                        TT.bestVal = T.meanPrice;
                                        TT.best = Some(broker.to_string());
                                        TT.meanPrice = T.meanPrice;
                                    }
                                    TT.transactions.insert(broker.to_string(), T);
                                }

                                println!("{} bought {} {} {}", broker, quantityBought, meanPrice, budres);
                            }
                            None => {}
                        }
                    }
                }
            }
        } else {}
    }
    if TT.best.is_some() {
        TT.best_recap = format!("{} {}@{} at {}", TT.typ, TT.symbol, TT.best.as_ref().unwrap(), TT.meanPrice);
    }

    TT
}

pub fn optimize_single_sell(budget: f64, infra: String, supra: String, R: &DataRegistry, DICT: &DictRegistry) -> Transactions {
    let name = "Single Sell".to_string();
    let mut TT = Transactions { typ: "SELL".to_string(), meanPrice: 0., best_recap: "".to_string(), name: format!("Sell {}/{}", supra, infra), symbol: format!("{}{}", supra, infra), transactions: HashMap::new(), bestVal: 0., best: None };
    for i in 0..BROKERS.len() {//for each broker
        let broker: &str = BROKERS[i];
        let RB = R.get(broker).unwrap();
        if infra == "USD" {
            if let Ok(D) = DICT.read() { //open dictionary to find broker name of the pair
                let DD: &Dictionary = &*D;
                let pairopt = DD.infrasupraToRawName(broker, &infra, &supra);
                if pairopt.is_some() {//if pair exists
                    let pair = pairopt.unwrap();
                    if let Ok(hm) = RB.read() { //read registry for this pair
                        let dataOption: Option<&RegistryData> = hm.get(&pair.to_string());
                        match dataOption {
                            Some(data) => {
                                let commissionBrokerTrading = 0.001;
                                let budgetAvailable = budget * (1. - commissionBrokerTrading);
                                let BIDS = data.get_bids();
                                let ordered: Vec<(f64, String, f64)> = orderbook_to_ordered(BIDS, true);
                                let mut budres = budgetAvailable;
                                let mut qtyres = 0.;
                                let mut quantitySold = 0.;
                                let mut meanPrice = 0.;
                                let mut earnings = 0.;
                                let mut T = Transaction {
                                    broker: broker.to_string(),
                                    budget: budget,
                                    commission:0.,
                                    commission_transfer:0.,
                                    tradingBudget: budgetAvailable,
                                    orders: Vec::new(),
                                    meanPrice: 0.,
                                    quantityTotal: 0.,
                                    value: 0.,
                                    remainer: budget,
                                };

                                for &(ref price, ref pricestr, ref size) in ordered.iter() {
                                    if budres <= 0.000000001 { break }
                                    println!("    {} Budget {}", broker, budres);
                                    let levelQty = *size;
                                    let levelPrice = *price;
                                    let mut operationQuantitySold;

                                    let sellable = budres;
                                    if levelQty > sellable {
                                        operationQuantitySold = budres;
                                    } else {
                                        operationQuantitySold = levelQty;
                                    }
                                    operationQuantitySold=operationQuantitySold;
                                    let operationEarnings = operationQuantitySold * levelPrice;
                                    earnings = earnings + operationEarnings;
                                    T.orders.push(Level { qty: operationQuantitySold, price: levelPrice, value: operationEarnings });
                                    quantitySold = quantitySold + operationQuantitySold;
                                    meanPrice = meanPrice + levelPrice * operationQuantitySold;
                                    budres = budres - operationQuantitySold;
                                    println!("    {} sell {} {} {} {} ", broker, levelQty, levelPrice, operationEarnings, operationQuantitySold);
                                }
                                T.commission=earnings * commissionBrokerTrading;
                                T.commission_transfer=get_deposit_commission(broker.to_string(),earnings);
                                meanPrice = meanPrice / quantitySold;
                                T.quantityTotal = quantitySold;
                                let residual = earnings-T.commission_transfer-T.commission;
                                T.meanPrice = meanPrice;
                                T.remainer = budres;

                                T.value = residual;


                                let mut active = T.quantityTotal > 0.;
                                if active {
                                    if T.meanPrice > TT.bestVal {
                                        TT.bestVal = T.meanPrice;
                                        TT.best = Some(broker.to_string());
                                        TT.meanPrice = T.meanPrice;
                                    }
                                    TT.transactions.insert(broker.to_string(), T);
                                }
                                println!("{} sold {} {} {}", broker, quantitySold, meanPrice, budres);
                            }
                            None => {}
                        }
                    }
                }
            }
        } else {}
    }
    if TT.best.is_some() {
        TT.best_recap = format!("{} {}@{} at {}", TT.typ, TT.symbol, TT.best.as_ref().unwrap(), TT.meanPrice);
    }
    TT
}

fn get_trading_commission(broker: String, value: f64) -> f64 {
    match broker.as_ref() {
        "binance" => {
            0.005 * value
        }
        _ => {
            0.01 * value
        }
    }
}

fn get_deposit_commission(broker: String, value: f64) -> f64 {
    match broker.as_ref() {
        "binance" => {
            0.
        }
        "hitbtc" => {
            0.
        }
        _ => {
            0.
        }
    }
}

fn get_withdraw_commission(broker: String, symbol: String, value: f64) -> f64 {
    match broker.as_ref() {
        "binance" => {
            match symbol{
                BNB=>0.92,
                BTC=>0.001 ,
                NEO=>0. ,
                ETH=>0.01,
                LTC=>0.01,
                QTUM=>0.01,
                EOS=>1.,
                SNT=>39. ,
                BNT=>1.7 ,
                GAS=>0. ,
                BCC=>0.001,
                BTM=>5. ,
                USDT=>17.1,
                HCC=>0.0005,
                HSR=>0.0001 ,
                OAX=>12.4 ,
                DNT=>104. ,
                MCO=>1.17 ,
                ICN=>5.3 ,
                ZRX=>7.8 ,
                OMG=>0.69 ,
                WTC=>0.4 ,
                LRC=>13. ,
                LLT=>67.8,
                YOYO=>52. ,
                TRX=>178.  ,
                STRAT=>0.1 ,
                SNGLS=>59. ,
                BQX=>2.1 ,
                KNC=>2.7 ,
                SNM=>38. ,
                FUN=>150. ,
                LINK=>20.5,
                XVG=>0.1 ,
                CTR=>9.5 ,
                SALT=>2. ,
                MDA=>6.5 ,
                IOTA=>0.5 ,
                SUB=>12.2 ,
                ETC=>0.01 ,
                MTL=>2.2 ,
                MTH=>57. ,
                ENG=>3. ,
                AST=>13.9,
                DASH=>0.002 ,
                BTG=>0.001 ,
                EVX=>4.9 ,
                REQ=>30.8 ,
                VIB=>35. ,
                POWR=>11.4,
                ARK=>0.1 ,
                XRP=>0.25 ,
                MOD=>3. ,
                ENJ=>58. ,
                STORJ=>8.6,
                VEN=>2. ,
                KMD=>0.002 ,
                RCN=>48. ,
                NULS=>3.4,
                RDN=>3.2 ,
                XMR=>0.04 ,
                DLT=>28.4 ,
                AMB=>15.9 ,
                BAT=>24. ,
                ZEC=>0.005,
                BCPT=>18. ,
                ARN=>5.3 ,
                GVT=>0.59 ,
                CDT=>92. ,
                GXS=>0.3 ,
                POE=>148. ,
                QSP=>30. ,
                BTS=>1. ,
                XZC=>0.02,
                LSK=>0.1 ,
                TNT=>59. ,
                FUEL=>71. ,
                MANA=>76.,
                BCD=>1. ,
                DGD=>0.04,
                ADX=>5.8,
                ADA=>1. ,
                PPT=>0.33,
                CMT=>47. ,
                XLM=>0.01,
                CND=>48. ,
                LEND=>100. ,
                WABI=>5.5 ,
                SBTC=>1.,
                BCX=>1.,
                WAVES=>0.002,
                TNB=>118. ,
                GTO=>35. ,
                ICX=>2.1 ,
                OST=>30. ,
                ELF=>7. ,
                AION=>3.1,
                ETF=>1. ,
                BRD=>10.5 ,
                NEBL=>0.01 ,
                VIBE=>18.7 ,
                LUN=>0.46 ,
                CHAT=>31.5,
                RLC=>6.1 ,
                INS=>3.8 ,
                IOST=>214.6 ,
                STEEM=>0.01,
                NANO=>0.01,
                AE=>3.2 ,
                VIA=>0.01,
                BLZ=>14. ,
                EDO=>4.3 ,
                WINGS=>13.7,
                NAV=>0.2 ,
                TRIG=>9.1 ,
                APPC=>13.5 ,
                PIVX=>0.02,
            }
        }
        "hitbtc" => {
            match symbol {
                BTC => 0.00085,
                BCC => 0.0018,
                ETH => 0.00215,
                ETC => 0.002,
                USDT => 100.,
                STRAT => 0.01,
                LTC => 0.003,
                DASH => 0.03,
                XMR => 0.09,
                BCN => 0.1,
                ARDR => 1.,
                STEEM => 0.01,
            }
        }
        _ => {
            0.
        }
    }
}