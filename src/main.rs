mod config;

use std::collections::HashMap;
use std::env::VarError;
use std::hash::Hash;
use std::time::{SystemTime, UNIX_EPOCH};
use reqwest::{Error, Response};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use crate::config::config::load_config;



const MINUTE_MS: u128 = 60000;
const HOUR_MS: u128 = 3600000;
const DAY_MS: u128 = 86400000;
#[derive(Debug, Deserialize)]
struct AnalystConfig {
    indicators: HashMap<String, HashMap<String, Indicator>>,
    candles: CandlesConfig,
    markets: MarketConfig
}

#[derive(Debug, Deserialize)]
struct MarketConfig {
    stocks: Vec<String>,
    currencies: Vec<String>
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

struct DataManager {
    
}



struct Granularity {
    period: String,
    multiplier: i32
}

#[derive(Debug, Serialize, Deserialize)]
struct PolygonAPIResponseCandle {
    c: f64,
    h: f64,
    l: f64,
    n: i64,
    o: f64,
    t: u128,
    v: f64,
    vw: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct PolygonAPIResponse {
    adjusted: bool,
    query_count: i128,
    request_id: String,
    results: Vec<PolygonAPIResponseCandle>,
    results_count: i128,
    status: String,
    ticker: String
}

fn granularity_converter(granularity: &str) -> Granularity {
    match granularity {
        "1m" => Granularity {
            period: "minute".to_string(),
            multiplier: 1
        },
        "5m" => Granularity {
            period: "minute".to_string(),
            multiplier: 5
        },
        "15m" => Granularity {
            period: "minute".to_string(),
            multiplier: 15
        },
        "30m" => Granularity {
            period: "minute".to_string(),
            multiplier: 30
        },
        "1h" => Granularity {
            period: "hour".to_string(),
            multiplier: 1
        },
        "1d" => Granularity {
            period: "day".to_string(),
            multiplier: 1
        },
        _ => Granularity {
            period: "day".to_string(),
            multiplier: 1
        }
    }
}




fn load_env(key: String) -> Result<String, VarError> {
    let value = std::env::var(key);
    match value {
        Ok(value) => Ok(value),
        Err(e) => Err(e)
    }
}

fn format_currency_ticker_for_polygon(currency: &str) -> String {
    let rem = currency.to_string().replace("-", "");
    format!("X:{}", rem.to_uppercase())
}


struct MarketDataMap {
    ticker: String,
    opens: Option<Vec<f64>>,
    highs: Option<Vec<f64>>,
    lows: Option<Vec<f64>>,
    closes: Option<Vec<f64>>,
    volume: Option<Vec<f64>>,
    indicators: Option<HashMap<String, Vec<f64>>>
}

impl MarketDataMap {
    fn new(ticker: String) -> Self {
        Self {
            ticker,
            opens: None,
            highs: None,
            lows: None,
            closes: None,
            volume: None,
            indicators: None
        }
    }
    fn add_indicators(&mut self, indicators: HashMap<String, Vec<f64>>) {
        self.indicators = Some(indicators);
    }
    fn add_indicator(&mut self, indicator_name: String, indicator_data: Vec<f64>) {
        if self.indicators.is_none() {
            self.indicators = Some(HashMap::new());
        }
        self.indicators.as_mut().unwrap().insert(indicator_name, indicator_data);
    }
    fn add_closes(&mut self, closes: Vec<f64>) {
        self.closes = Some(closes);
    }
    
    fn add_opens(&mut self, opens: Vec<f64>) {
        self.opens = Some(opens);
    }
    fn add_highs(&mut self, highs: Vec<f64>) {
        self.highs = Some(highs);
    }
    fn add_lows(&mut self, lows: Vec<f64>) {
        self.lows = Some(lows);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let analyst_toml = load_config("analyst.toml");
    let analyst_config: AnalystConfig  = toml::from_str(&analyst_toml).expect("Error parsing config");
    let granularity = granularity_converter(&analyst_config.candles.interval);
    let polygon_api_key = std::env::var("POLYGON_API_KEY").expect("POLYGON_API_KEY not set");


    // Fetch market candles and store in hashmap
    let mut candle_data_map: HashMap<String, Vec<f64>> = HashMap::new();
    for market in analyst_config.markets.currencies {
        // calculate the datetime range for candles
        let to_dt = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        let from_dt: u128 = to_dt - (30 * DAY_MS);
        // Format Ticker
        let ticker = format_currency_ticker_for_polygon(&market);
        // Format URL
        let url = format!("https://api.polygon.io/v2/aggs/ticker/{}/range/{}/{}/{}/{}?adjusted=true&sort=asc&limit={}",&ticker, &granularity.multiplier, &granularity.period, from_dt, to_dt, analyst_config.candles.limit);
        // Make HTTP Request to Polygon API
        let client = reqwest::Client::new();
        let candles_res = client.get(&url)
            .header(CONTENT_TYPE, "application/json")
            .header(AUTHORIZATION, format!("Bearer {}", &polygon_api_key))
            .send().await.expect("Error fetching candles");

        match candles_res.status()  {
            reqwest::StatusCode::OK => {
                match candles_res.json::<PolygonAPIResponse>().await { 
                    Ok(candles_data) => {
                        println!("Candles Data: {:?}", candles_data);
                        candle_data_map.insert(market, candles_data.results.iter().map(|c| c.c).collect());
                    }
                    Err(e) => {
                        println!("Error parsing candles data: {}", e);
                    }
                }
            }
            reqwest::StatusCode::UNAUTHORIZED => {
                
            }
            other => {
                println!("Error fetching candles: {}", other);
            }
        }
    }

    // iterate over indicators in analyst_config
    for (indicator_name, indicator_config) in analyst_config.indicators {
        match indicator_name.as_str() {
            "sma" => {
                println!("{} ::> {:?}", &indicator_name, &indicator_config);
            },
            "ema" => {
                println!("{} ::> {:?}", &indicator_name, &indicator_config);
            },
            "rsi" => {
                println!("{} ::> {:?}", &indicator_name, &indicator_config);
            },
            "macd" => {
                println!("{} ::> {:?}", &indicator_name, &indicator_config);
            },
            "bollinger_bands" => {
                println!("{} ::> {:?}", &indicator_name, &indicator_config);
            }
            _ => {
                println!("Incorrectly formatted indicator")
            }
        }
    }
    Ok(())
}
