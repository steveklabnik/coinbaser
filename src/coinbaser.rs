use std::io::Error as IoError;
#[allow(unused_imports)] use std::io::{self, Read};
use hyper::client::Client;
use hyper::status::StatusCode;
use hyper::Url;

use error::Error as _CoinError;

pub const ENDPOINT: &'static str = "https://api.exchange.coinbase.com";
pub const _SANDBOX_ENDPOINT: &'static str = "https://api-public.sandbox.exchange.coinbase.com";

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
