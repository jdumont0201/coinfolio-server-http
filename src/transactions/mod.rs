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

use structures::{Portfolio, Transaction, Level, TransactionResult, OptimizationResult, Operations, Transactions};

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

pub fn getSellTransaction(ptf: &Portfolio, data: &RegistryData, broker: String, infra: String, supra: String, useOrderbook: bool) -> TransactionResult {
    let commissionBrokerTrading = get_trading_commission_pc(broker.to_string());
    let budgetAvailable = ptf.qty;
    let mut BIDS = data.get_bids();
    let mut ordered: Vec<(f64, String, f64)>=Vec::new();
    if useOrderbook {
        let BIDS = data.get_asks();
        ordered = orderbook_to_ordered(BIDS, true);
    } else {
        //use fake unlimited orderbook for approximation mode
        let b = data.bid.clone();
        match b {
            Some(b_) => {
               // println!("bidval {}",b_);
                let val = b_.parse::<f64>();
                match val {
                    Ok(val_) => {
                 //        println!("parseok {}",val_);
                        ordered = vec![(val_, b_.to_string(), 10000000000000.)];
                    }
                    Err(err) => {
                        println!("err parsing val {}s", b_);
                    }
                }
            }
            None => { println!("nothing iin option sell") }
        }
    }

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
//println!("bidres {} {:?}",budres,ordered);
    for &(ref price, ref pricestr, ref size) in ordered.iter() {
        if budres <= 0.000000001 { break }
        let levelQty = *size;
        let levelPrice = *price;
        let mut operationQuantitySold;
       // println!("    sell iter nb p{} q{} b{}",levelPrice,levelQty,budres);

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
    if quantitySold<0.0000001 {
        println!("err no sold")
    }
    meanPrice = meanPrice / quantitySold;
    T.quantityTotal = quantitySold;
    let residual = earnings - T.commission;
    T.meanPrice = meanPrice;
    T.remainer = budres;

    T.value = residual;
    TransactionResult { portfolio: Portfolio { qty: residual, value: T.value, asset: infra }, remainer: Portfolio { qty: T.remainer, value: T.remainer * ptf.value / ptf.qty, asset: supra }, transaction: T }
}

pub fn getBuyTransaction(ptf: &Portfolio, data: &RegistryData, broker: String, infra: String, supra: String, useOrderbook: bool) -> TransactionResult {
    let commissionBrokerTrading = get_trading_commission_pc(broker.to_string());
    let budgetAvailable = ptf.value;
    let mut ordered: Vec<(f64, String, f64)>=Vec::new();
    if useOrderbook {
        let ASKS = data.get_asks();
        ordered = orderbook_to_ordered(ASKS, false);
    } else {
        //use fake unlimited orderbook for approximation mode
        let b = data.ask.clone();
        match b {
            Some(b_) => {
                let val = b_.parse::<f64>();
                match val {
                    Ok(val_) => {
                        ordered = vec![(val_, b_.to_string(), 10000000000000.)];
                    }
                    Err(err) => {
                        println!("err parsing val {}s", b_);
                    }
                }
            }
            None => { println!("nothing iin option buy") }
        }
    }
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
   // println!("  buy ordered {:?}", ordered);
    for &(ref price, ref pricestr, ref size) in ordered.iter() {
      //  println!("    buy iter p{} s{} {}", price, size,budres);
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
