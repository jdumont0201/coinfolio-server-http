use Data;
use std::collections::HashMap;

static NAME:&str="bitfinex";
pub static URL_HTTP_BIDASK:&str="https://api.bitfinex.com/v2/tickers?symbols=tBTCUSD,tLTCUSD,tLTCBTC,tETHUSD,tETHBTC,tETCBTC,tETCUSD,tRRTUSD,tRRTBTC,tZECUSD,tZECBTC,tXMRUSD,tXMRBTC,tDSHUSD,tDSHBTC,tBTCEUR,tXRPUSD,tXRPBTC,tIOTUSD,tIOTBTC,tIOTETH,tEOSUSD,tEOSBTC,tEOSETH,tSANUSD,tSANBTC,tSANETH,tOMGUSD,tOMGBTC,tOMGETH,tBCHUSD,tBCHBTC,tBCHETH,tNEOUSD,tNEOBTC,tNEOETH,tETPUSD,tETPBTC,tETPETH,tQTMUSD,tQTMBTC,tQTMETH,tAVTUSD,tAVTBTC,tAVTETH,tEDOUSD,tEDOBTC,tEDOETH,tBTGUSD,tBTGBTC,tDATUSD,tDATBTC,tDATETH,tQSHUSD,tQSHBTC,tQSHETH,tYYWUSD,tYYWBTC,tYYWETH,tGNTUSD,tGNTBTC,tGNTETH,tSNTUSD,tSNTBTC,tSNTETH,tIOTEUR,tBATUSD,tBATBTC,tBATETH,tMNAUSD,tMNABTC,tMNAETH,tFUNUSD,tFUNBTC,tFUNETH,tZRXUSD,tZRXBTC,tZRXETH,tTNBUSD,tTNBBTC,tTNBETH,tSPKUSD,tSPKBTC,tSPKETH,tTRXUSD,tTRXBTC,tTRXETH,tRCNUSD,tRCNBTC,tRCNETH,tRLCUSD,tRLCBTC,tRLCETH,tAIDUSD,tAIDBTC,tAIDETH,tSNGUSD,tSNGBTC,tSNGETH,tREPUSD,tREPBTC,tREPETH,tELFUSD,tELFBTC,tELFETH";

pub fn parse_bidask(text:String) -> HashMap<String,Data>{
    let mut r = HashMap::new();
    let text2b = str::replace(&text, "[[", "");
    let text2 = str::replace(&text2b, "]]", "");
    let mut bs: Vec<&str> = text2.split("],[").collect();
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