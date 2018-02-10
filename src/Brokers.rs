use std;

pub static BROKERS: &'static [&str] = &["binance", "hitbtc", "kucoin", "kraken", "cryptopia", "bitfinex"];

pub enum TASK {
    HTTP_PRICE,
    HTTP_BIDASK,
    HTTP_DEPTH,
    WS_TICK,
    WS_TRADE,
    WS_DEPTH,
}

#[derive(Clone,Copy,Debug)]
pub enum BROKER {
    BINANCE,
    BITFINEX,
    KRAKEN,
    KUCOIN,
    CRYPTOPIA,
    HITBTC,
}

impl std::fmt::Display for BROKER {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn getKey(broker: BROKER) -> String {
    match broker {
        BROKER::BINANCE => "binance".to_string(),
        BROKER::KRAKEN => "kraken".to_string(),
        BROKER::KUCOIN => "kucoin".to_string(),
        BROKER::BITFINEX => "bitfinex".to_string(),
        BROKER::CRYPTOPIA => "cryptopia".to_string(),
        BROKER::HITBTC => "hitbtc".to_string(),
    }
}

pub fn getEnum(broker: String) -> Option<BROKER> {
    match broker.as_ref() {
        "binance" => Some(BROKER::BINANCE),
        "kraken" => Some(BROKER::KRAKEN),
        "kucoin" => Some(BROKER::KUCOIN),
        "bitfinex" => Some(BROKER::BITFINEX),
        "cryptopia" => Some(BROKER::CRYPTOPIA),
        "hitbtc" => Some(BROKER::HITBTC),
        _ => None
    }
}
