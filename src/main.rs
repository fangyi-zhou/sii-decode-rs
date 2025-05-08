use std::env;
use std::fs;

use simple_logger::SimpleLogger;

mod bsii_file;
mod bsii_output;
mod bsii_parse;
mod file_type;
mod scsc_file;
mod scsc_parse;

fn main() {
    SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()
        .unwrap();

    let arg = env::args().next_back().unwrap();
    let content = fs::read(arg).unwrap();
    let decoded = file_type::decode_until_siin(&content).unwrap();
    println!("{}", String::from_utf8(decoded).unwrap());
}
