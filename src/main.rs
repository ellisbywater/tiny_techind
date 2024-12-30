mod config;

use std::collections::HashMap;
use std::hash::Hash;
use serde::{Deserialize, Serialize};
use crate::config::config::load_config;


#[derive(Debug, Deserialize)]
struct AnalystConfig {
    indicators: HashMap<String, HashMap<String, Indicator>>,
    candles: CandlesConfig,
    markets: HashMap<String, MarketConfig>
}

#[derive(Debug, Deserialize)]
struct MarketConfig {
    base: String,
    quote: String,
}

#[derive(Debug, Deserialize)]
struct CandlesConfig {
    interval: String,
    limit: i32,
}

#[derive(Debug, Deserialize)]
struct IndicatorConfig {
    indicator: HashMap<String, Indicator>
}

#[derive(Debug, Deserialize)]
struct Indicator {
    period: i32,
    field: String,
    std_dev: i32,
    fast_period: i32,
    slow_period: i32,
    signal_period: i32,
}



fn main() {
    let analyst_toml = load_config("analyst.toml");
    let analyst_config: AnalystConfig  = toml::from_str(&analyst_toml).expect("Error parsing config");
    // println!("Hello, world!");
    println!("{:#?}", &analyst_config);
    // iterate over indicators in analyst_config
    for (indicator_name, indicator_config) in analyst_config.indicators {
        println!("{}  {:#?}",&indicator_name, &indicator_config);
        
        match indicator_name.as_str() {  
            "sma" => {
                
            },
            "ema" => {
                println!("{:?}", &indicator_config);
            },
            _ => {
                
            }
        }
    }
}
