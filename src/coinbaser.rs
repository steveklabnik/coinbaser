#[allow(dead_code)]

use std::io::Error as IoError;
#[allow(unused_imports)] use std::io::{self, Read};
use chrono::{DateTime, UTC};
use hyper::client::Client;
use hyper::status::StatusCode;
use hyper::Url;
use uuid::Uuid;

use error::Error as _CoinError;

pub const ENDPOINT: &'static str = "https://api.exchange.coinbase.com";
pub const _SANDBOX_ENDPOINT: &'static str = "https://api-public.sandbox.exchange.coinbase.com";

pub type Price = f64;

enum HttpGetError {
	/// Unable to parse a provided URL
	BadUrl(String),
	/// HTTP error response, along with the server response string
	BadStatus(StatusCode, String),
	Io(IoError),
	/// Internal (Hyper) error
	Internal(String)
}

/// Plain function to fetch a string from a URL.
fn http_get(path: &str) -> Result<String, HttpGetError> {
	let url = match Url::parse(path.clone()) {
		Ok(url) => url,
		Err(e) => {
			return Err(HttpGetError::BadUrl(format!("{}", e)));
		}
	};

	let mut client = Client::new();
	let request = client.get(url);
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
#[derive(Debug)]
pub struct HistoricRate {
	pub time: DateTime<UTC>,
	pub low: Price,
	pub high: Price,
	pub open: Price,
	pub close: Price,
	pub volume: f64
}

///https://docs.exchange.coinbase.com/#get-24hr-stats
#[derive(Debug)]
pub struct DayStat {
	pub open: Price,
	pub high: Price,
	pub low: f64,
	pub volume: f64,
}

///https://docs.exchange.coinbase.com/#currencies
#[derive(Debug, Clone)]
pub struct Currency {
	pub id: String,
	pub name: String,
	pub min_size: Price
}