use debug;
use std;
use time::Duration;
use Brokers::{BROKER, getEnum, TASK, BROKERS};
use dictionary::Dictionary;
use dictionary;
use Universal::{Universal_Orderbook, RegistryData};
use types::{DataRegistry, TextRegistry, DictRegistry, OrderbookSide, BidaskRegistry, BidaskReadOnlyRegistry, BidaskTextRegistry};
use routes::orderbook_to_ordered;
use std::collections::HashMap;
use chrono::offset::{TimeZone, Utc};
use commissions::{get_trading_commission_pc, get_deposit_commission, get_withdraw_commission};
use transactions::{getDepositTransaction, getSellTransaction, getBuyTransaction, getWithdrawTransaction};
use structures::{Portfolio, Transaction, Level, TransactionResult, OptimizationResult, Operations, Transactions};

// for each broker, reads data[PAIR] and computes cheapest ask and most expensive bid
pub fn recap(budget: f64, infra: String, supra: String, R: &DataRegistry, DICT: &DictRegistry) -> String {
    let mut OS = optimize_single(budget, infra, supra, R, DICT);
    OS.to_json()
}

pub fn approx(R: &DataRegistry, DICT: &DictRegistry) -> String {
    let mut res: Vec<Operations> = Vec::new();
    let mut restxt: String = "".to_string();
    if let Ok(D) = DICT.read() {
        let uniToRaw = &D.uniToRaw;
        for (infra, list) in uniToRaw {
            for (supra, ref brokers) in list {
                for (broker_name, data) in brokers.iter() {
                    // println!("try {} {} {}", infra, supra, broker_name);
                    let broker_e = getEnum(broker_name.to_string()).unwrap();
                    let pairopt = dictionary::read_rawname(broker_e, supra.to_string(), infra.to_string(), DICT);
                    if pairopt.is_some() {
                        let pair = pairopt.unwrap();
                        let broker_name_str: &str = &broker_name;
                        let RB = R.get(broker_name_str).unwrap();
                        if let Ok(hm) = RB.read() {
                            //read registry for this pair
                            let dataOption: Option<&RegistryData> = hm.get(&pair.to_string().to_uppercase());
                            match dataOption {
                                Some(data) => {
                                    for (broker2_name, data2) in brokers.iter() {
                                        if broker_name != broker2_name {
                                            let broker2: &str;
                                            let broker2_e = getEnum(broker2_name.to_string()).unwrap();
                                            let pairopt2 = dictionary::read_rawname(broker2_e, supra.to_string(), infra.to_string(), DICT);
                                            if pairopt2.is_some() {
                                                let pair2 = pairopt2.unwrap();
                                                let broker2_name_str: &str = &broker2_name;
                                                let RB2 = R.get(broker2_name_str).unwrap();
                                                if let Ok(hm2) = RB2.read() {
                                                    //read registry for this pair
                                                    let dataOption2: Option<&RegistryData> = hm2.get(&pair2.to_string().to_uppercase());
                                                    match dataOption2 {
                                                        Some(data2) => {
                                                            if data.bid.is_some() && data2.ask.is_some() {
                                                                if infra.to_string() == "USD".to_string() {
                                                                    let O = approx_broker_USD(infra.to_string(), supra.to_string(), broker_name.to_string(), pair.to_string(), data, broker2_name.to_string(), pair2, data2);
                                                                    if (O.profitpc > 0.) {
                                                                        println!("profit {}", O.name);
                                                                        res.push(O);
                                                                    }
                                                                } else {
                                                                    let pairusdopt = dictionary::read_rawname(broker_e,infra.to_string(), "USD".to_string(), DICT);
                                                                    if pairusdopt.is_some() {
                                                                        let pairusd = pairusdopt.unwrap();
                                                                        let dataOption_usd: Option<&RegistryData> = hm.get(&pairusd.to_string().to_uppercase());
                                                                        match dataOption_usd {
                                                                            Some(datausd) => {
                                                                                let pair2usdopt = dictionary::read_rawname(broker2_e, infra.to_string(), "USD".to_string(), DICT);
                                                                                if pair2usdopt.is_some() {
                                                                                    let pair2usd = pair2usdopt.unwrap();
                                                                                    let dataOption_usd2: Option<&RegistryData> = hm2.get(&pair2usd.to_string().to_uppercase());
                                                                                    match dataOption_usd2 {
                                                                                        Some(datausd2) => {
                                                                                            let O = approx_broker(infra.to_string(),
                                                                                                                  supra.to_string(),
                                                                                                                  broker_name.to_string(),
                                                                                                                  pair.to_string(),
                                                                                                                  data,
                                                                                                                  datausd,
                                                                                                                  broker2_name.to_string(),
                                                                                                                  pair2,
                                                                                                                  data2,
                                                                                                                  datausd2);
                                                                                            if (O.profitpc > 0.) {
                                                                                                println!("profit {}", O.name);
                                                                                                datausd.print();
                                                                                                data.print();
                                                                                                data2.print();
                                                                                                datausd2.print();
                                                                                                res.push(O);
                                                                                            }
                                                                                        }
                                                                                        None=>{}
                                                                                    }
                                                                                }
                                                                            }None=>{}
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }

                                                        None => { debug::warn(format!("  No data for  broker2 {}", broker2_name)) }
                                                    }
                                                } else { debug::warn(format!("  cannot read hm2 {}{}{}", broker2_name, infra, supra)) }
                                            } else { debug::warn(format!("  cannot read dictionary2 {}{}{}", broker2_name, infra, supra)) }
                                        } else {}
                                    }
                                }
                                None => { debug::warn(format!("  No data for first broker {}", broker_name)) }
                            }
                        } else { debug::warn(format!("  cannot read hm1 {}{}{}", broker_name, infra, supra)) }
                    } else { debug::warn(format!("  cannot read dictionary {}{}{}", broker_name, infra, supra)) }
                }
            }
        }
    }
    //output
    res.sort_by(|b, a| a.profit.partial_cmp(&b.profit).unwrap());
    let mut st: String = "[".to_string();
    for item in res.iter() {
        restxt = format!("{}{}{}", restxt, st, item.to_json());
        st = ",".to_string();
    }
    format!("{}]", restxt)
}

pub fn approx_broker(infra: String, supra: String, broker: String, pair: String, data: &RegistryData, datausd: &RegistryData, broker2: String, pair2: String, data2: &RegistryData, datausd2: &RegistryData) -> Operations {
    //  println!("----------------------------");
    let budget = 1000.;
    let origin = Portfolio { qty: budget, asset: "USD".to_string(), value: budget };
    let T0 = getBuyTransaction(&origin, datausd, broker.to_string(), "USD".to_string(), infra.to_string(), false);
    let T1 = getBuyTransaction(&T0.clone().portfolio, data, broker.to_string(), infra.to_string(), supra.to_string(), false);
    //println!("T1 q,{}v={}", T1.portfolio.qty, T1.portfolio.value);
    let T2 = getWithdrawTransaction(&T1.clone().portfolio, data, broker.to_string(), infra.to_string(), supra.to_string());
    let T3 = getDepositTransaction(&T2.clone().portfolio, data2, broker2.to_string(), infra.to_string(), supra.to_string());
    let T4 = getSellTransaction(&T3.clone().portfolio, data2, broker2.to_string(), infra.to_string(), supra.to_string(), false);
    let T5 = getSellTransaction(&T4.clone().portfolio, datausd2, broker2.to_string(), "USD".to_string(), infra.to_string(), false);
    let profit = T4.portfolio.value - budget;
    let profitpc = profit / budget * 100.;
    let title = format!("{}@{} at {} /  {}@{} at {} = {}", pair.to_string(), broker, T1.transaction.meanPrice, pair2, broker2, T4.transaction.meanPrice, profitpc);
    // println!("  -> COMP {}", title);
    let mut o = Operations { operations: vec![T0.clone(),T1.clone(), T2.clone(), T3.clone(), T4.clone(),T5.clone()], recap: Vec::new(), name: title.to_string(), profit: profit, profitpc: profitpc };
    o.set_recap();
    o
}

pub fn approx_broker_USD(infra: String, supra: String, broker: String, pair: String, data: &RegistryData, broker2: String, pair2: String, data2: &RegistryData) -> Operations {
    //  println!("----------------------------");
    let budget = 1000.;
    let origin = Portfolio { qty: budget, asset: "USD".to_string(), value: budget };
    let T1 = getBuyTransaction(&origin, data, broker.to_string(), infra.to_string(), supra.to_string(), false);
    //println!("T1 q,{}v={}", T1.portfolio.qty, T1.portfolio.value);
    let T2 = getWithdrawTransaction(&T1.clone().portfolio, data, broker.to_string(), infra.to_string(), supra.to_string());
    let T3 = getDepositTransaction(&T2.clone().portfolio, data2, broker2.to_string(), infra.to_string(), supra.to_string());
    let T4 = getSellTransaction(&T3.clone().portfolio, data2, broker2.to_string(), infra.to_string(), supra.to_string(), false);
    let profit = T4.portfolio.value - budget;
    let profitpc = profit / budget * 100.;
    let title = format!("{}@{} at {} /  {}@{} at {} = {}", pair.to_string(), broker, T1.transaction.meanPrice, pair2, broker2, T4.transaction.meanPrice, profitpc);
    // println!("  -> COMP {}", title);
    let mut o = Operations { operations: vec![T1.clone(), T2.clone(), T3.clone(), T4.clone()], recap: Vec::new(), name: title.to_string(), profit: profit, profitpc: profitpc };
    o.set_recap_usd();
    o
}

pub fn optimize_single(budget: f64, infra: String, supra: String, R: &DataRegistry, DICT: &DictRegistry) -> OptimizationResult {
    let mut O = Operations { operations: Vec::new(), recap: Vec::new(), name: "Buy sell".to_string(), profit: -10000000., profitpc: 0. };
    let mut H = HashMap::new();
    for i in 0..BROKERS.len() {
        //for each broker
        //let mut TT = Transactions { typ: "BUY".to_string(), meanPrice: 0., best_recap: "".to_string(), name: format!("Buy {}/{}", supra.to_string(), infra.to_string()), symbol: format!("{}{}", supra, infra), transactions: HashMap::new(), bestVal: 1000000000., best: None };
        if infra == "USD" {
            let broker: &str = BROKERS[i];
            println!("try {}", broker);
            let broker_e = getEnum(BROKERS[i].to_string()).unwrap();
            let pairopt = dictionary::read_rawname(broker_e, supra.to_string(), infra.to_string(), DICT);
            if pairopt.is_some() {
                //if pair exists
                println!("   try {} is some", broker);
                let pair = pairopt.unwrap();
                let RB = R.get(broker).unwrap();
                println!("{} READ {}", broker, pair);
                if let Ok(hm) = RB.read() {
                    //read registry for this pair
                    println!("try {} read ok", broker);
                    let dataOption: Option<&RegistryData> = hm.get(&pair.to_string().to_uppercase());
                    match dataOption {
                        Some(data) => {
                            if data.has_asks() {
                                debug::print_read_depth(broker_e, &pair.to_string(), &format!(" first broker ok"));
                                let mut HH = HashMap::new();
                                let origin = Portfolio { qty: budget, asset: "USD".to_string(), value: budget };
                                let T1 = getBuyTransaction(&origin, data, broker.to_string(), infra.to_string(), supra.to_string(), true);
                                let T2 = getWithdrawTransaction(&T1.clone().portfolio, data, broker.to_string(), infra.to_string(), supra.to_string());
                                for j in 0..BROKERS.len() {
                                    //for each broker
                                    if i != j {
                                        let broker2: &str = BROKERS[j];
                                        println!("   try {}", broker2);
                                        let broker2_e = getEnum(BROKERS[j].to_string()).unwrap();
                                        let RB2 = R.get(broker2).unwrap();
                                        if let Ok(hm2) = RB2.read() {
                                            //read registry for this pair
                                            println!("   try {} read ok", broker2);
                                            let pair2opt = dictionary::read_rawname(broker2_e, supra.to_string(), infra.to_string(), DICT);
                                            if pair2opt.is_some() {
                                                //if pair exists
                                                println!("   try {} is some", broker2);
                                                let pair2 = pair2opt.unwrap();
                                                let dataOption2: Option<&RegistryData> = hm2.get(&pair2.to_string().to_uppercase());
                                                match dataOption2 {
                                                    Some(data2) => {
                                                        println!("   try {} has data", broker2);
                                                        if data2.has_bids() {
                                                            println!("   try {} has bids", broker2);
                                                            debug::print_read_depth(broker2_e, &pair.to_string(), &format!("second broker ok"));
                                                            let T3 = getDepositTransaction(&T2.clone().portfolio, data2, broker2.to_string(), infra.to_string(), supra.to_string());
                                                            let T4 = getSellTransaction(&T3.clone().portfolio, data2, broker2.to_string(), infra.to_string(), supra.to_string(), true);
                                                            let profit = T4.portfolio.value - budget;
                                                            let profitpc = profit / budget * 100.;
                                                            let mut o = Operations { operations: vec![T1.clone(), T2.clone(), T3.clone(), T4.clone()], recap: Vec::new(), name: "Buy sell".to_string(), profit: profit, profitpc: profitpc };
                                                            o.set_recap();

                                                            HH.insert(broker2.to_string(), o);
                                                            //println!("insert {} {}", broker, broker2);
                                                            if profit > O.profit {
                                                                //println!("update best ");
                                                                O.profit = profit;
                                                                O.profitpc = profitpc;
                                                                O.operations = vec![T1.clone(), T2.clone(), T3.clone(), T4.clone()];
                                                                O.set_recap();
                                                                ;
                                                            }
                                                            println!("{} -> {} = {}", broker, broker2, profit);
                                                        } else {
                                                            debug::print_read_depth(broker_e, &pair.to_string(), &format!(" second broker nok"));
                                                        }
                                                    }
                                                    None => {
                                                        println!("   try {} has no data in hm for {}", broker2, &pair2.to_string().to_uppercase());
                                                    }
                                                }
                                            } else { println!("   try {} no match in dict for ", broker2); }
                                        }
                                    }
                                }
                                H.insert(broker.to_string(), HH);
                            } else {
                                debug::print_read_depth(broker_e, &pair.to_string(), &format!(" first broker nok"));
                            }
                        }
                        None => {}
                    }
                }
            }
        }
    }
    OptimizationResult { best: O, all: H }
}
