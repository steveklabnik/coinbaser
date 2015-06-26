extern crate hyper;

mod coinbaser;
pub mod error;

use coinbaser::*;

fn main() {
	println!("{:?}", ENDPOINT);
}