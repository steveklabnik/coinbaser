#[allow(dead_code)]

use std::collections::HashMap;
use std::error::Error;
use std::io::Error as IoError;
#[allow(unused_imports)] use std::io::{self, Read};
use chrono::{DateTime, UTC};
use hyper::client::Client;
use hyper::header::{Headers, UserAgent};
use hyper::status::StatusCode;
use hyper::Url;
use serde::{Deserialize, Deserializer};
use serde::de::Visitor;
use uuid::Uuid;

use error::Error as _CoinError;

pub const DEFAULT_ENDPOINT: &'static str = "https://api.exchange.coinbase.com";
pub const DEFAULT_SANDBOX_ENDPOINT: &'static str = "https://api-public.sandbox.exchange.coinbase.com";

pub type Price = f64;

#[derive(Debug)]
pub enum HttpGetError {
	/// Unable to parse a provided URL
	BadUrl(String),
	/// HTTP error response, along with the server response string
	BadStatus(StatusCode, String),
	Io(IoError),
	/// Internal (Hyper) error
	Internal(String)
}

/// Plain function to fetch a string from a URL.
pub fn http_get(path: &str, agent: &str) -> Result<String, HttpGetError> {
	let url = match Url::parse(path.clone()) {
		Ok(url) => url,
		Err(e) => {
			return Err(HttpGetError::BadUrl(format!("{}", e)));
		}
	};

	let client = Client::new();
	let request = client.get(url);

	let mut headers = Headers::new();
	headers.set(UserAgent(agent.into()));
	let request = request.headers(headers);

	let mut response = match request.send() {
		Ok(res) => res,
		Err(e) => {
			return Err(HttpGetError::Internal(e.to_string()));
		}
	};

	let mut data = String::new();
	let read_result = response.read_to_string(&mut data);
	let data = match read_result {
		Ok(_) => data,
		Err(e) => {
			return Err(HttpGetError::Io(e));
		}
	};

	if response.status == StatusCode::Ok {
		Ok(data)
	} else {
		Err(HttpGetError::BadStatus(response.status, data))
	}
}

/// Used to validate data when converting from JSON dummy structures.
struct BuilderState {
	pub currencies: HashMap<String, Currency>
}

impl BuilderState {
	/// Searches for a currency with an abbreviation of `key`
	pub fn try_currency(&self, key: &str) -> Option<Currency> {
		for (id, cur) in &self.currencies {
			if id == key {
				return Some(Currency {
					id: key.to_string(),
					name: cur.name.clone(),
					min_size: cur.min_size
				});
			}
		}
		None
	}
}

/// Converter from a dummy struct to validated data
trait Builder<F, E> {
	fn build(state: &BuilderState, from: F) -> Result<Self, E>;
}

/// Occurs on bad JSON
pub enum ValidationError {
	BadCurrency(String),
	BadDecimal(String)
}

///https://docs.exchange.coinbase.com/#products
#[derive(Debug)]
pub struct Product {
	pub id: (Currency, Currency),
	pub base_currency: Currency,
	pub quote_currency: Currency,
	pub base_min_size: Price,
	pub base_max_size: Price,
	pub quote_increment: Price
}

#[derive(Debug, Deserialize)]
struct DummyProduct {
	pub id: String,
	pub base_currency: String,
	pub quote_currency: String,
	pub base_min_size: String,
	pub base_max_size: String,
	pub quote_increment: String
}

fn get_price(key: &str) -> Result<Price, ValidationError> {
	key.parse().map_err(|_| ValidationError::BadDecimal("".to_string()))
}
impl Builder<DummyProduct, ValidationError> for Product {
	fn build(state: &BuilderState, from: DummyProduct) -> Result<Product, ValidationError> {
		// does the provided currency ID exist from what we've loaded?
		fn get_curr(state: &BuilderState, key: Option<&&str>) -> Result<Currency, ValidationError> {
			if let Some(key) = key {
				match state.try_currency(key.clone()) {
					Some(cur) => Ok(cur),
					None => Err(ValidationError::BadCurrency(format!("Unknown currency {}", key)))
				}
			} else {
				// key was passed as None which means that the string split failed
				Err(ValidationError::BadCurrency("Parse error".to_string()))
			}
		}
		// split the "BASE-QUOTE" format into two
		let key: Vec<&str> = from.id.split("-").collect();
		let (base, quote) = (
			try!(get_curr(state, key.get(0))),
			try!(get_curr(state, key.get(1)))
		);
		// build
		Ok(Product {
			id: (base.clone(), quote.clone()),
			base_currency: base,
			quote_currency: quote,
			base_min_size: try!(get_price(&from.base_min_size)),
			base_max_size: try!(get_price(&from.base_max_size)),
			quote_increment: try!(get_price(&from.quote_increment))
		})
	}
}

/// https://docs.exchange.coinbase.com/#get-product-order-book
#[derive(Debug, Clone, Copy)]
pub struct Order {
	pub price: Price,
	pub size: Price,
	pub num_orders: Option<i32>,
	pub id: Option<Uuid>
}


/// https://docs.exchange.coinbase.com/#get-product-order-book
pub struct OrderBook {
	pub bids: Vec<Order>,
	pub asks: Vec<Order>
}

///https://docs.exchange.coinbase.com/#get-product-ticker
#[derive(Debug)]
pub struct Ticker {
	pub trade_id: i32,
	pub price: Price,
	pub size: Price,
	pub time: DateTime<UTC>
}

#[derive(Debug)]
pub enum TradeSide {
	Buy,
	Sell
}
pub use self::TradeSide::*;

///https://docs.exchange.coinbase.com/#get-trades
#[derive(Debug)]
pub struct Trade {
	pub time: DateTime<UTC>,
	pub trade_id: i64,
	pub price: Price,
	pub size: Price,
	pub side: TradeSide
}

///https://docs.exchange.coinbase.com/#get-historic-rates
#[derive(Debug, Deserialize)]
pub struct HistoricRate {
	pub time: DateTime<UTC>,
	pub low: Price,
	pub high: Price,
	pub open: Price,
	pub close: Price,
	pub volume: f64
}

///https://docs.exchange.coinbase.com/#get-24hr-stats
#[derive(Debug, Deserialize)]
pub struct DayStat {
	pub open: Price,
	pub high: Price,
	pub low: f64,
	pub volume: f64,
}

///https://docs.exchange.coinbase.com/#currencies
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Currency {
	pub id: String,
	pub name: String,
	pub min_size: Price
}