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
use commissions::{get_trading_commission_pc,get_deposit_commission,get_withdraw_commission};
#[derive(Clone)]
pub struct Portfolio {
    pub qty: f64,
    pub asset: String,
    pub value: f64,
}

impl Portfolio {
    fn print(&self) {
        println!("{} {} = {}", self.qty, self.asset, self.value)
    }
    fn to_json(&self) -> String {
        format!("{{\"qty\":{},\"asset\":\"{}\",\"value\":{} }}", self.qty, self.asset, self.value)
    }
}

pub struct OptimizationResult {
    best: Operations,
    all: HashMap<String, HashMap<String, Operations>>,
}

impl OptimizationResult {
    fn to_json(&mut self) -> String {
        let best;
        //if self.best.profit > 0. { best = self.best.to_json() } else { best = "null".to_string() };
        best = self.best.to_json();

        let mut ordersstr = format!("{{\"best\":{},\"all\":{{", best);
        let mut st = "";
        for (br1, list) in &self.all {
            if list.len() > 0 {
                ordersstr = format!("{}{}\"{}\":{{", ordersstr, st, br1);
                let mut st2 = "";
                for (br2, o) in list {
                    ordersstr = format!("{}{} \"{}\":{}", ordersstr, st2, br2, o.to_json());
                    st2 = ",";
                }
                st = ",";
                ordersstr = format!("{} }}", ordersstr);
            }
        }
        ordersstr = format!("{} }} }}", ordersstr);
        ordersstr
    }
}

pub struct Operations {
    operations: Vec<TransactionResult>,
    name: String,
    profit: f64,
    profitpc: f64,
    recap: Vec<String>,
}

impl Operations {
    fn set_recap(&mut self) {
        let T1 = &self.operations[0];
        let T2 = &self.operations[1];
        let T3 = &self.operations[2];
        let T4 = &self.operations[3];
        self.recap =
            vec![
                format!("{} {}{}@{} at {}", T1.transaction.typ, T1.transaction.supra, T1.transaction.infra, T1.transaction.broker, T1.transaction.meanPrice),
                format!("{} {}{}@{}", T2.transaction.typ, T2.transaction.supra, T2.transaction.infra, T2.transaction.broker),
                format!("{} {}{}@{}", T3.transaction.typ, T3.transaction.supra, T3.transaction.infra, T3.transaction.broker),
                format!("{} {}{}@{} at {}", T4.transaction.typ, T4.transaction.supra, T4.transaction.infra, T4.transaction.broker, T4.transaction.meanPrice),
            ];
    }
    fn to_json(&self) -> String {
        let mut total_com = 0.;
        for i in self.operations.iter() {
            total_com = total_com + i.transaction.commission;
        }
        let mut ordersstr = format!("{{\"profit\":\"{}\",\"profit_pc\":\"{}\",\"total_commission\":{},\"detail\":[", self.profit, self.profitpc, total_com);
        let mut st = "";
        for i in self.recap.iter() {
            ordersstr = format!("{}{}\"{}\"", ordersstr, st, i);
            st = ",";
        }
        ordersstr = format!("{}],\"operations\":[", ordersstr);
        let mut st = "";
        let mut total_com = 0.;
        for i in self.operations.iter() {
            ordersstr = format!("{}{}{{\"transaction\":{},\"result\":{},\"remainer\":{} }}", ordersstr, st, i.transaction.to_json(), i.portfolio.to_json(), i.remainer.to_json());
            st = ",";
        }
        ordersstr = format!("{}]}}", ordersstr);
        ordersstr
    }
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

#[derive(Clone)]
pub struct Transaction {
    broker: String,
    typ: String,
    budget: f64,
    commission: f64,
    tradingBudget: f64,
    orders: Vec<Level>,
    meanPrice: f64,
    infra: String,
    supra: String,

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

        format!("{{ \"type\":\"{}\", \"broker\":\"{}\",\"budget\":{}, \"commission\":{},\"orders\":{},\"meanPrice\":{},\"quantityExchanged\":{},\"remainer\":{} }}", self.typ, self.broker, self.budget, self.commission, ordersstr, self.meanPrice, self.quantityTotal, self.remainer)
    }
}

#[derive(Clone)]
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
    let mut OS = optimize_single(budget, infra, supra, R, DICT);
    OS.to_json()
}


pub fn optimize_single(budget: f64, infra: String, supra: String, R: &DataRegistry, DICT: &DictRegistry) -> OptimizationResult {
    let mut O = Operations { operations: Vec::new(), recap: Vec::new(), name: "Buy sell".to_string(), profit: -10000000., profitpc: 0. };
    let mut H = HashMap::new();
    for i in 0..BROKERS.len() {//for each broker
        //let mut TT = Transactions { typ: "BUY".to_string(), meanPrice: 0., best_recap: "".to_string(), name: format!("Buy {}/{}", supra.to_string(), infra.to_string()), symbol: format!("{}{}", supra, infra), transactions: HashMap::new(), bestVal: 1000000000., best: None };
        if infra == "USD" {
            let broker: &str = BROKERS[i];
            println!("try {}", broker);
            let broker_e = getEnum(BROKERS[i].to_string()).unwrap();
            let pairopt = dictionary::read_rawname(broker_e, supra.to_string(), infra.to_string(), DICT);
            if pairopt.is_some() {//if pair exists
                println!("   try {} is some", broker);
                let pair = pairopt.unwrap();
                let RB = R.get(broker).unwrap();
                println!("{} READ {}", broker, pair);
                if let Ok(hm) = RB.read() { //read registry for this pair
                    println!("try {} read ok", broker);
                    let dataOption: Option<&RegistryData> = hm.get(&pair.to_string().to_uppercase());
                    match dataOption {
                        Some(data) => {
                            if data.has_bids() {
                                debug::print_read_depth(broker_e, &pair.to_string(), &format!(" first broker ok"));

                                let mut HH = HashMap::new();
                                let origin = Portfolio { qty: budget, asset: "USD".to_string(), value: budget };
                                let T1 = getBuyTransaction(&origin, data, broker.to_string(), infra.to_string(), supra.to_string());
                                let T2 = getWithdrawTransaction(&T1.clone().portfolio, data, broker.to_string(), infra.to_string(), supra.to_string());
                                for j in 0..BROKERS.len() {//for each broker

                                    if i != j {
                                        let broker2: &str = BROKERS[j];
                                        println!("   try {}", broker2);
                                        let broker2_e = getEnum(BROKERS[j].to_string()).unwrap();
                                        let RB2 = R.get(broker2).unwrap();
                                        if let Ok(hm2) = RB2.read() { //read registry for this pair
                                            println!("   try {} read ok", broker2);
                                            let pair2opt = dictionary::read_rawname(broker2_e, supra.to_string(), infra.to_string(), DICT);
                                            if pair2opt.is_some() {//if pair exists
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
                                                            let T4 = getSellTransaction(&T3.clone().portfolio, data2, broker2.to_string(), infra.to_string(), supra.to_string());

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

#[derive(Clone)]
pub struct TransactionResult {
    pub portfolio: Portfolio,
    pub remainer: Portfolio,
    pub transaction: Transaction,
}

pub fn getDepositTransaction(ptf: &Portfolio, data: &RegistryData, broker: String, infra: String, supra: String) -> TransactionResult {
    let mut T = Transaction {
        broker: broker.to_string(),
        budget: ptf.qty,
        commission: 0.,
        typ: "DEPOSIT".to_string(),
        tradingBudget: ptf.qty,
        orders: Vec::new(),
        infra: infra.to_string(),
        supra: supra.to_string(),
        meanPrice: 0.,
        value: 0.,
        quantityTotal: 0.,
        remainer: 0.,
    };
    T.commission = get_deposit_commission(broker.to_string(), ptf.qty);
    let q = ptf.qty - T.commission;
    let v = ptf.value * q / ptf.qty;
    TransactionResult { portfolio: Portfolio { qty: q, value: v, asset: supra.to_string() }, remainer: Portfolio { qty: 0., value: 0., asset: supra }, transaction: T }
}

pub fn getWithdrawTransaction(ptf: &Portfolio, data: &RegistryData, broker: String, infra: String, supra: String) -> TransactionResult {
    let mut T = Transaction {
        broker: broker.to_string(),
        budget: ptf.qty,
        commission: 0.,
        typ: "WITHDRAW".to_string(),
        infra: infra.to_string(),
        supra: supra.to_string(),
        tradingBudget: ptf.qty,
        orders: Vec::new(),
        meanPrice: 0.,
        value: 0.,
        quantityTotal: 0.,
        remainer: 0.,
    };
    let commission_supra = get_withdraw_commission(broker.to_string(), ptf.asset.to_string(), ptf.qty);
    let ASKS = data.get_asks();
    let ordered: Vec<(f64, String, f64)> = orderbook_to_ordered(ASKS, false);
    //println!("or {}",ordered.len());
    if ordered.len() > 0 {
        let priceapprx = ordered[0].0;
        //  println!("or {:?}",ordered[0]);
        T.commission = commission_supra * priceapprx;
    }
    let q = ptf.qty - commission_supra;
    let v = ptf.value - T.commission;//* q / ptf.qty;
    TransactionResult { portfolio: Portfolio { qty: q, value: v, asset: supra.to_string() }, remainer: Portfolio { qty: 0., value: 0., asset: supra }, transaction: T }
}

pub fn getSellTransaction(ptf: &Portfolio, data: &RegistryData, broker: String, infra: String, supra: String) -> TransactionResult {
    let commissionBrokerTrading = get_trading_commission_pc(broker.to_string());
    let budgetAvailable = ptf.qty;
    let BIDS = data.get_bids();
    let ordered: Vec<(f64, String, f64)> = orderbook_to_ordered(BIDS, true);
    let mut budres = budgetAvailable;
    let mut qtyres = 0.;
    let mut quantitySold = 0.;
    let mut meanPrice = 0.;
    let mut earnings = 0.;
    let mut T = Transaction {
        broker: broker.to_string(),
        budget: ptf.qty,
        typ: "SELL".to_string(),
        commission: 0.,

        tradingBudget: budgetAvailable,
        orders: Vec::new(),
        meanPrice: 0.,
        infra: infra.to_string(),
        supra: supra.to_string(),
        quantityTotal: 0.,
        value: 0.,
        remainer: 0.,
    };

    for &(ref price, ref pricestr, ref size) in ordered.iter() {
        if budres <= 0.000000001 { break }
        let levelQty = *size;
        let levelPrice = *price;
        let mut operationQuantitySold;

        let sellable = budres;
        if levelQty > sellable {
            operationQuantitySold = budres;
        } else {
            operationQuantitySold = levelQty;
        }
        operationQuantitySold = operationQuantitySold;
        let operationEarnings = operationQuantitySold * levelPrice;

        earnings = earnings + operationEarnings;
        //println!("  op {} earning {} {}",broker,operationEarnings,earnings);
        T.orders.push(Level { qty: operationQuantitySold, price: levelPrice, value: operationEarnings });
        quantitySold = quantitySold + operationQuantitySold;
        meanPrice = meanPrice + levelPrice * operationQuantitySold;
        budres = budres - operationQuantitySold;
    }
    T.commission = earnings * commissionBrokerTrading;

    meanPrice = meanPrice / quantitySold;
    T.quantityTotal = quantitySold;
    let residual = earnings - T.commission;
    T.meanPrice = meanPrice;
    T.remainer = budres;

    T.value = residual;
    TransactionResult { portfolio: Portfolio { qty: residual, value: T.value, asset: infra }, remainer: Portfolio { qty: T.remainer, value: T.remainer * ptf.value / ptf.qty, asset: supra }, transaction: T }
}

pub fn getBuyTransaction(ptf: &Portfolio, data: &RegistryData, broker: String, infra: String, supra: String) -> TransactionResult {
    let commissionBrokerTrading = get_trading_commission_pc(broker.to_string());
    let budgetAvailable = ptf.value;
    let ASKS = data.get_asks();
    let ordered: Vec<(f64, String, f64)> = orderbook_to_ordered(ASKS, false);
    let mut budres = budgetAvailable;
    let mut qtyres = 0.;
    let mut quantityBought = 0.;
    let mut meanPrice = 0.;
    let mut invested = 0.;
    let mut T = Transaction {
        broker: broker.to_string(),
        budget: ptf.qty,
        typ: "BUY".to_string(),
        infra: infra.to_string(),
        supra: supra.to_string(),
        commission: 0.,
        tradingBudget: budgetAvailable,
        orders: Vec::new(),
        meanPrice: 0.,
        value: 0.,
        quantityTotal: 0.,
        remainer: 0.,
    };
    for &(ref price, ref pricestr, ref size) in ordered.iter() {
        if budres <= 0.000000001 { break }
        let levelQty = *size;
        let levelPrice = *price;
        let mut operationQuantityBought;
        let buyable = budres / levelPrice;
        if levelQty > buyable {
            operationQuantityBought = buyable;
        } else {
            operationQuantityBought = levelQty;
        }
        operationQuantityBought = operationQuantityBought;//commission
        let operationCost = operationQuantityBought * levelPrice;
        T.orders.push(Level { qty: operationQuantityBought, price: levelPrice, value: operationCost });
        invested = invested + operationCost;
        quantityBought = quantityBought + operationQuantityBought;
        // meanPrice = meanPrice + levelPrice * operationQuantityBought;
        budres = budres - operationCost;
    }
    T.commission = invested * commissionBrokerTrading;
    let residual = invested - T.commission;
    meanPrice = invested / quantityBought; //meanPrice / quantityBought;
    T.quantityTotal = quantityBought;
    T.meanPrice = meanPrice;
    T.remainer = budres;
    T.value = T.remainer + residual;
    T.commission = T.value * commissionBrokerTrading;
    TransactionResult { portfolio: Portfolio { qty: T.quantityTotal, value: T.value, asset: supra }, remainer: Portfolio { qty: T.remainer, value: T.remainer, asset: infra }, transaction: T }
}
