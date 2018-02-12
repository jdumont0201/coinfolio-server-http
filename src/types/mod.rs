use dictionary::{Dictionary,generateReference};
use std::collections::HashMap;
use std::sync::{RwLock ,Arc, Mutex};
use std::cell::RefCell;
use Universal::{Universal_Orderbook,RegistryData};
//TYPES FOR SHARED STRUCTURES ACROSS THREADS
pub type DataRegistry = HashMap<String,Arc<RwLock<HashMap<String, RegistryData>>>>;
pub type TextRegistry = HashMap<String,Arc<RwLock<String>>>;
pub type DictRegistry = Arc<RwLock<Dictionary>>;
pub type OrderbookSide = HashMap<String,f64>;
pub type BidaskRegistry = Arc<Mutex<Option<HashMap<String, HashMap<String, RegistryData>>>>>;
pub type BidaskReadOnlyRegistry = Arc<RwLock<Option<HashMap<String, HashMap<String, RegistryData>>>>>;
pub type BidaskTextRegistry = Arc<Mutex<Option<HashMap<String, String>>>>;

