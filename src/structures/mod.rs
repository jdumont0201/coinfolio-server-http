use std::collections::HashMap;
use Universal::{Universal_Orderbook, RegistryData};
use types::{DataRegistry, TextRegistry, DictRegistry, OrderbookSide, BidaskRegistry, BidaskReadOnlyRegistry, BidaskTextRegistry};


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


#[derive(Clone)]
pub struct TransactionResult {
    pub portfolio: Portfolio,
    pub remainer: Portfolio,
    pub transaction: Transaction,
}

pub struct OptimizationResult {
    pub best: Operations,
    pub all: HashMap<String, HashMap<String, Operations>>,
}

impl OptimizationResult {
    pub fn to_json(&mut self) -> String {
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
    pub  operations: Vec<TransactionResult>,
    pub name: String,
    pub    profit: f64,
    pub   profitpc: f64,
    pub    recap: Vec<String>,
}

impl Operations {
    pub fn isProfitable(&self) -> bool {
        self.profitpc > 0.
    }
    pub fn set_recap_usd(&mut self) {
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
    pub fn set_recap(&mut self) {
        let T0 = &self.operations[0];
        let T1 = &self.operations[1];
        let T2 = &self.operations[2];
        let T3 = &self.operations[3];
        let T4 = &self.operations[4];
        let T5 = &self.operations[5];
        self.recap =
            vec![
                format!("{} {}{}@{} at {}", T0.transaction.typ, T0.transaction.supra, T0.transaction.infra, T0.transaction.broker, T0.transaction.meanPrice),
                format!("{} {}{}@{} at {}", T1.transaction.typ, T1.transaction.supra, T1.transaction.infra, T1.transaction.broker, T1.transaction.meanPrice),
                format!("{} {}{}@{}", T2.transaction.typ, T2.transaction.supra, T2.transaction.infra, T2.transaction.broker),
                format!("{} {}{}@{}", T3.transaction.typ, T3.transaction.supra, T3.transaction.infra, T3.transaction.broker),
                format!("{} {}{}@{} at {}", T4.transaction.typ, T4.transaction.supra, T4.transaction.infra, T4.transaction.broker, T4.transaction.meanPrice),
                format!("{} {}{}@{} at {}", T5.transaction.typ, T5.transaction.supra, T5.transaction.infra, T5.transaction.broker, T5.transaction.meanPrice),
            ];
    }
    pub fn to_json(&self) -> String {
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
    pub  transactions: HashMap<String, Transaction>,
    pub   bestVal: f64,
    pub   best: Option<String>,
    pub   best_recap: String,
    pub   name: String,
    pub   symbol: String,

    pub   typ: String,
    pub   meanPrice: f64,
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
    pub   broker: String,
    pub   typ: String,
    pub budget: f64,
    pub   commission: f64,
    pub    tradingBudget: f64,
    pub    orders: Vec<Level>,
    pub    meanPrice: f64,
    pub    infra: String,
    pub   supra: String,

    pub  remainer: f64,
    pub    quantityTotal: f64,
    pub   value: f64,
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
pub struct Level {
    pub   qty: f64,
    pub   price: f64,
    pub   value: f64,
}

impl Level {
    pub fn to_json(&self) -> String {
        format!("{{\"qty\":{},\"price\":{},\"cost\":{} }}", self.qty, self.price, self.value)
    }
}

