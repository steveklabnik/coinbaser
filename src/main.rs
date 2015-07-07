#![feature(custom_derive)]

extern crate chrono;
extern crate hyper;
extern crate serde;
extern crate uuid;

mod coinbaser;
pub mod error;

use coinbaser::*;

fn main() {
	// println!("{:?}", ENDPOINT);
	println!("{:?}", http_get("https://api.exchange.coinbase.com/products", "hyper/0.6.0 TESTER"));
}