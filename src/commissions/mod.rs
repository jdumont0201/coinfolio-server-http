
fn get_trading_commission(broker: String, value: f64) -> f64 {
    get_trading_commission_pc(broker) * value
}

pub fn get_trading_commission_pc(broker: String) -> f64 {
    match broker.as_ref() {
        "binance" => {
            0.0005
        }
        "kucoin" => {
            0.001
        }
        "bitfinex" => {
            0.002
        }
        _ => {
            0.001
        }
    }
}

pub fn get_deposit_commission(broker: String, value: f64) -> f64 {
    match broker.as_ref() {
        "binance" => {
            0.
        }
        "kucoin" => {
            0.
        }
        "hitbtc" => {
            0.
        }
        "bitfinex" => {
            0.
        }
        _ => {
            0.
        }
    }
}

pub fn get_withdraw_commission(broker: String, symbol: String, value: f64) -> f64 {
    match broker.as_ref() {
        "bitfinex" => {
            match symbol.as_ref() {
                "BTC" => { 0.0008 },
                "ETH" => { 0.0027  },
                "XRP" => { 0.02 },/*,
            Bitcoin Cash    0.0001 BCH
            Litecoin    0.001 LTC
            Eos    0.30541 EOS
            Neo    FREE
            Iota    0.5 IOTA
            Ethereum Classic    0.01 ETC
            Zcash    0.001 ZEC
            Monero    0.04 XMR
            Dash    0.01 DASH
            Omise Go    0.18622 OMG
            Bitcoin Gold    FREE
            Santiment    1.2489 SAN
            Qtum    0.01 QTUM
            Tron    55.971 TRX
            0x    1.8884 ZRX
            Etp    0.01 ETP
            Qash    2.5201 QASH
            Streamr    21.222 DATA
            Aelf    1.9395 ELF
            Fun Fair    43.903 FUN
            Status    12.274 SNT
            Eidoo    0.99738 EDO
            Yoyow    14.686 YYW
            Decentraland    18.896 MNA
            Spank Chain    8.0084 SPK
            Aid Coin    8.0284 AID
            Time New Bank    31.881 TNB
            Golem    6.1203 GNT
            Basic Attention Token    7.0313 BAT
            Augur    0.042024 REP
            Aventus    0.86262 AVT
            Rcn    12.469 RCN
            I Exec    1.7015 RLC
            Singular Dtv    16.654 SNG
            TetherUSD(Omni)    20.0 USD
            TetherUSD(Ethereum ERC20)    2.2354 USD
            TetherUSD(Ethereum ERC20)    1.796 EUR
            Bank wire    0.100 % (min 20 USD / Euro)
            Express bank wire(within 24 hours on business days)    1.000 % (min 20 USD / Euro)*/
                _=>{0.}
            }
        }
        "kucoin" => {
            match symbol.as_ref() {
                "KCS" => { 2. }
                "BTC" => { 0.001 }
                "USDT" => { 10. }
                "ETH" => { 0.01 }
                "ACAT" => { 1. }
                "CTR" => { 2. }
                "LTC" => { 0.001 }
                "NEO" => { 0. }
                "HAT" => { 0.5 }
                "GAS" => { 0. }
                "KNC" => { 0.5 }
                "BTM" => { 5. }
                "QTUM" => { 0.1 }
                "EOS" => { 0.5 }
                "CVC" => { 3. }
                "OMG" => { 0.1 }
                "PAY" => { 0.5 }
                "SNT" => { 20. }
                "BHC" => { 1. }
                "HSR" => { 0.01 }
                "WTC" => { 0.1 }
                "VEN" => { 2. }
                "MTH" => { 10. }
                "RPX" => { 1. }
                "REQ" => { 20. }
                "EVX" => { 0.5 }
                "MOD" => { 0.5 }
                "NEBL" => { 0.1 }
                "DGB" => { 0.5 }
                "CAG" => { 2. }
                "CFD" => { 0.5 }
                "RDN" => { 0.5 }
                "UKG" => { 5. }
                "BCPT" => { 5. }
                "PPT" => { 0.1 }
                "BCH" => { 0.0005 }
                "STX" => { 2. }
                "NULS" => { 1. }
                "GVT" => { 0.1 }
                "HST" => { 2. }
                "PURA" => { 0.5 }
                "SUB" => { 2. }
                "QSP" => { 5. }
                "POWR" => { 1. }
                "FLIXX" => { 10. }
                "LEND" => { 20. }
                "AMB" => { 3. }
                "RHOC" => { 2. }
                "R" => { 2. }
                "DENT" => { 50. }
                "DRGN" => { 1. }
                "ACT" => { 0.1 }
                "ENJ" => { 10. }
                "CAT" => { 20. }
                "DAT" => { 20. }
                "CL" => { 50. }
                "TEL" => { 500. }
                "DNA" => { 3. }
                "AGI" => { 2. }
                "COFI" => { 5. }
                "ARY" => { 10. }
                "cV" => { 30. }
                "ZPT" => { 1. }
                "EBTC" => { 3. }
                "ING" => { 3. }
                "HPB" => { 0.5 }
                "CXO" => { 30. }
                "TKY" => { 1. }
                "COV" => { 3. }
                "PARETO" => { 40. }
                "MWAT" => { 20. }
                _ => { 0. }
            }
        }
        "binance" => {
            match symbol.as_ref() {
                "BNB" => { 0.92 }
                "BTC" => { 0.001 }
                "NEO" => { 0. }
                "ETH" => { 0.01 }
                "LTC" => { 0.01 }
                "QTUM" => { 0.01 }
                "EOS" => { 1. }
                "SNT" => { 39. }
                "BNT" => { 1.7 }
                "GAS" => { 0. }
                "BCC" => { 0.001 }
                "BTM" => { 5. }
                "USDT" => { 17.1 }
                "HCC" => { 0.0005 }
                "HSR" => { 0.0001 }
                "OAX" => { 12.4 }
                "DNT" => { 104. }
                "MCO" => { 1.17 }
                "ICN" => { 5.3 }
                "ZRX" => { 7.8 }
                "OMG" => { 0.69 }
                "WTC" => { 0.4 }
                "LRC" => { 13. }
                "LLT" => { 67.8 }
                "YOYO" => { 52. }
                "TRX" => { 178. }
                "STRAT" => { 0.1 }
                "SNGLS" => { 59. }
                "BQX" => { 2.1 }
                "KNC" => { 2.7 }
                "SNM" => { 38. }
                "FUN" => { 150. }
                "LINK" => { 20.5 }
                "XVG" => { 0.1 }
                "CTR" => { 9.5 }
                "SALT" => { 2. }
                "MDA" => { 6.5 }
                "IOTA" => { 0.5 }
                "SUB" => { 12.2 }
                "ETC" => { 0.01 }
                "MTL" => { 2.2 }
                "MTH" => { 57. }
                "ENG" => { 3. }
                "AST" => { 13.9 }
                "DASH" => { 0.002 }
                "BTG" => { 0.001 }
                "EVX" => { 4.9 }
                "REQ" => { 30.8 }
                "VIB" => { 35. }
                "POWR" => { 11.4 }
                "ARK" => { 0.1 }
                "XRP" => { 0.25 }
                "MOD" => { 3. }
                "ENJ" => { 58. }
                "STORJ" => { 8.6 }
                "VEN" => { 2. }
                "KMD" => { 0.002 }
                "RCN" => { 48. }
                "NULS" => { 3.4 }
                "RDN" => { 3.2 }
                "XMR" => { 0.04 }
                "DLT" => { 28.4 }
                "AMB" => { 15.9 }
                "BAT" => { 24. }
                "ZEC" => { 0.005 }
                "BCPT" => { 18. }
                "ARN" => { 5.3 }
                "GVT" => { 0.59 }
                "CDT" => { 92. }
                "GXS" => { 0.3 }
                "POE" => { 148. }
                "QSP" => { 30. }
                "BTS" => { 1. }
                "XZC" => { 0.02 }
                "LSK" => { 0.1 }
                "TNT" => { 59. }
                "FUEL" => { 71. }
                "MANA" => { 76. }
                "BCD" => { 1. }
                "DGD" => { 0.04 }
                "ADX" => { 5.8 }
                "ADA" => { 1. }
                "PPT" => { 0.33 }
                "CMT" => { 47. }
                "XLM" => { 0.01 }
                "CND" => { 48. }
                "LEND" => { 100. }
                "WABI" => { 5.5 }
                "SBTC" => { 1. }
                "BCX" => { 1. }
                "WAVES" => { 0.002 }
                "TNB" => { 118. }
                "GTO" => { 35. }
                "ICX" => { 2.1 }
                "OST" => { 30. }
                "ELF" => { 7. }
                "AION" => { 3.1 }
                "ETF" => { 1. }
                "BRD" => { 10.5 }
                "NEBL" => { 0.01 }
                "VIBE" => { 18.7 }
                "LUN" => { 0.46 }
                "CHAT" => { 31.5 }
                "RLC" => { 6.1 }
                "INS" => { 3.8 }
                "IOST" => { 214.6 }
                "STEEM" => { 0.01 }
                "NANO" => { 0.01 }
                "AE" => { 3.2 }
                "VIA" => { 0.01 }
                "BLZ" => { 14. }
                "EDO" => { 4.3 }
                "WINGS" => { 13.7 }
                "NAV" => { 0.2 }
                "TRIG" => { 9.1 }
                "APPC" => { 13.5 }
                "PIVX" => { 0.02 }
                _ => { 0. }
            }
        }
        "hitbtc" => {
            match symbol.as_ref() {
                "BTC" => { 0.00085 }
                "BCC" => { 0.0018 }
                "ETH" => { 0.00215 }
                "ETC" => { 0.002 }
                "USDT" => { 100. }
                "STRAT" => { 0.01 }
                "LTC" => { 0.003 }
                "DASH" => { 0.03 }
                "XMR" => { 0.09 }
                "BCN" => { 0.1 }
                "ARDR" => { 1. }
                "STEEM" => { 0.01 }
                _ => { 0. }
            }
        }
        _ => {
            0.
        }
    }
}