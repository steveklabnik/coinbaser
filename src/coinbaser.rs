#[allow(dead_code)]

use std::error::Error;
use std::io::Error as IoError;
#[allow(unused_imports)] use std::io::{self, Read};
use chrono::{DateTime, UTC};
use hyper::client::Client;
use hyper::header::{Headers, UserAgent};
use hyper::status::StatusCode;
use hyper::Url;
use rustc_serialize::{Decodable};
use uuid::Uuid;

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

/// Passable state of the server
pub struct State {
	pub currencies: Vec<Currency>
}

impl State {
	pub fn get_curr(&self, key: &str) -> Option<&Currency> {
		for curr in &self.currencies {
			if curr.id == key {
				return Some(&curr);
			}
		}
		None
	}
}

/// Server result doesn't match state
pub enum ValidationError {
	BadCurrency(String)
}

///https://docs.exchange.coinbase.com/#products
#[derive(Debug, PartialEq)]
pub struct Product<'a> {
	pub id: (&'a Currency, &'a Currency),
	pub base_currency: &'a Currency,
	pub quote_currency: &'a Currency,
	pub base_min_size: Price,
	pub base_max_size: Price,
	pub quote_increment: Price
}

mod dummy_product {
	use super::{Currency, Price, Product, State, ValidationError};
	use std::convert::From;

	#[derive(Debug, RustcDecodable)]
	pub struct DummyProduct {
		pub id: String,
		pub base_currency: String,
		pub quote_currency: String,
		pub base_min_size: Price,
		pub base_max_size: Price,
		pub quote_increment: Price
	}

	impl <'a> Product<'a> {
		// checks if the provided currency ID exist from what we've loaded
		fn get_curr(state: &'a State, key: Option<&&str>) -> Result<&'a Currency, ValidationError> {
			if let Some(key) = key {
				match state.get_curr(key.clone()) {
					Some(cur) => Ok(cur),
					None => Err(ValidationError::BadCurrency(format!("Unknown currency `{}`, maybe refresh the list?", key)))
				}
			} else { // key was passed as None which means that the string split failed
				Err(ValidationError::BadCurrency("Parse error".to_string()))
			}
		}

		pub fn from_dummy(state: &'a State, p: DummyProduct) -> Result<Self, ValidationError> {
			// split the "BASE-QUOTE" format into two
			let key: Vec<&str> = p.id.split("-").collect();
			let (base, quote) = (
				try!(Product::get_curr(state, key.get(0))),
				try!(Product::get_curr(state, key.get(1)))
			);
			// build
			Ok(Product {
				id: (base, quote),
				base_currency: base,
				quote_currency: quote,
				base_min_size: p.base_min_size,
				base_max_size: p.base_max_size,
				quote_increment: p.quote_increment
			})
		}
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
#[derive(Debug)]
pub struct OrderBook {
	pub bids: Vec<Order>,
	pub asks: Vec<Order>
}

mod dummy_orderbook {
	use rustc_serialize::Decodable;
	use super::{Price, Order, OrderBook};
	use uuid::{ParseError, Uuid};
	#[derive(Debug, PartialEq, RustcDecodable)]
	pub struct DummyOrder(Price, Price, i32);
	#[derive(Debug, PartialEq, RustcDecodable)]
	pub struct DummyOrderLvl3(Price, Price, String);

	impl Order {
		pub fn from_order(o: DummyOrder) -> Self {
			Order {
				price: o.0,
				size: o.1,
				num_orders: Some(o.2),
				id: None
			}
		}

		pub fn from_order3(o: DummyOrderLvl3) -> Result<Self, ParseError> {
			use std::str::FromStr;
			Ok(Order {
				price: o.0,
				size: o.1,
				num_orders: None,
				id: Some(try!(Uuid::from_str(&o.2)))
			})
		}
	}

	#[derive(Debug, RustcDecodable)]
	pub struct DummyOrderBook {
		pub bids: Vec<DummyOrder>,
		pub asks: Vec<DummyOrder>
	}
	#[derive(Debug, RustcDecodable)]
	pub struct DummyOrderBookLvl3 {
		pub bids: Vec<DummyOrderLvl3>,
		pub asks: Vec<DummyOrderLvl3>
	}

	impl OrderBook {
		pub fn from_orderbook(d: DummyOrderBook) -> Self {
			OrderBook {
				bids: d.bids.into_iter().map(|b| Order::from_order(b)).collect(),
				asks: d.asks.into_iter().map(|b| Order::from_order(b)).collect()
			}
		}

		pub fn from_orderbook3(d: DummyOrderBookLvl3) -> Result<Self, ParseError> {
			let mut bids: Vec<Order> = Vec::new();
			for bid in d.bids {
				match Order::from_order3(bid) {
					Ok(bid) => bids.push(bid),
					Err(e) => { return Err(e); }
				}
			}
			let mut asks: Vec<Order> = Vec::new();
			for ask in d.asks {
				match Order::from_order3(ask) {
					Ok(ask) => asks.push(ask),
					Err(e) => { return Err(e); }
				}
			}
			Ok(OrderBook {
				bids: bids,
				asks: asks
			})
		}
	}
}

///https://docs.exchange.coinbase.com/#get-product-ticker
#[derive(Debug)]
pub struct Ticker {
	pub trade_id: i32,
	pub price: Price,
	pub size: Price,
	pub time: DateTime<UTC>
}

mod dummy_ticker {
	use chrono::format::ParseError;
	use rustc_serialize::Decodable;
	use super::{Price, Ticker};

	#[derive(Debug, RustcDecodable)]
	pub struct DummyTicker {
		pub trade_id: i32,
		pub price: Price,
		pub size: Price,
		pub time: String
	}

	impl Ticker {
		pub fn from_dummy(t: DummyTicker) -> Result<Self, ParseError> {
			use chrono::{DateTime, UTC};
			use std::str::FromStr;
			Ok(Ticker {
				trade_id: t.trade_id,
				price: t.price,
				size: t.size,
				time: try!(DateTime::<UTC>::from_str(&t.time))
			})
		}
	}
}

#[derive(Debug, RustcDecodable)]
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

mod dummy_trade {
	use chrono::{DateTime, UTC, ParseError};
	use rustc_serialize::Decodable;
	use super::{Price, Trade, TradeSide};

	#[derive(Debug, RustcDecodable)]
	pub struct DummyTrade {
		pub time: String,
		pub trade_id: i64,
		pub price: Price,
		pub size: Price,
		pub side: TradeSide
	}

	impl Trade {
		pub fn from_dummy(t: DummyTrade) -> Result<Self, ParseError> {
			use std::str::FromStr;
			Ok(Trade {
				time: try!(DateTime::<UTC>::from_str(&t.time)),
				trade_id: t.trade_id,
				price: t.price,
				size: t.size,
				side: t.side
			})
		}
	}
}

///https://docs.exchange.coinbase.com/#get-historic-rates
#[derive(Debug)]
pub struct HistoricRate {
	pub time: DateTime<UTC>,
	pub low: Price,
	pub high: Price,
	pub open: Price,
	pub close: Price,
	pub volume: f64
}

mod dummy_historic {
	use chrono::{DateTime, UTC, ParseError};
	use rustc_serialize::Decodable;
	use super::{Price, HistoricRate};

	#[derive(Debug, RustcDecodable)]
	pub struct DummyHistoricRate {
		pub time: String,
		pub low: Price,
		pub high: Price,
		pub open: Price,
		pub close: Price,
		pub volume: f64
	}

	impl HistoricRate {
		pub fn from_dummy(t: DummyHistoricRate) -> Result<Self, ParseError> {
			use std::str::FromStr;
			Ok(HistoricRate {
				time: try!(DateTime::<UTC>::from_str(&t.time)),
				low: t.low,
				high: t.high,
				open: t.open,
				close: t.close,
				volume: t.volume
			})
		}
	}
}

///https://docs.exchange.coinbase.com/#get-24hr-stats
#[derive(Debug, RustcDecodable)]
pub struct DayStat {
	pub open: Price,
	pub high: Price,
	pub low: f64,
	pub volume: f64,
}

///https://docs.exchange.coinbase.com/#currencies
#[derive(Debug, PartialEq, RustcDecodable)]
pub struct Currency {
	pub id: String,
	pub name: String,
	pub min_size: Price
}

pub fn tester() {
	use rustc_serialize::{json, Decoder};

	println!("{}/currencies", DEFAULT_ENDPOINT);
	let down = http_get(&format!("{}/currencies", DEFAULT_ENDPOINT), "hyper/0.6.0/coinbaser");
	let down = match down {
		Ok(res) => res,
		Err(e) => {
			println!("{:?}", e);
			return;
		}
	};
	let down = down.trim();
	let t: Currency = match json::decode(down) {
		Ok(j) => j,
		Err(e) => {
			println!("{:?}", e);
			return;
		}
	};

	println!("{:?}", t);
}