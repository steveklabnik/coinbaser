extern crate chrono;
extern crate hyper;
extern crate rustc_serialize;
extern crate uuid;

mod coinbaser;
pub mod error;

use coinbaser::*;

fn main() {
	// println!("{:?}", ENDPOINT);
	tester();
}