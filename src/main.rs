extern crate hyper;
extern crate chrono;
extern crate uuid;

mod coinbaser;
pub mod error;

use coinbaser::*;

fn main() {
	println!("{:?}", ENDPOINT);
}