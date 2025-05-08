use std::env;
use std::fs;

use bsii_file::BsiiFile;
use scsc_file::ScscFile;
use simple_logger::SimpleLogger;

mod bsii_file;
mod bsii_output;
mod bsii_parse;
mod scsc_file;
mod scsc_parse;

fn main() {
    SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()
        .unwrap();

    let arg = env::args().next_back().unwrap();
    let content = fs::read(arg).unwrap();
    let scsc_file = ScscFile::parse(content.as_slice()).unwrap();
    let bsii_binary = scsc_file.decode();
    fs::write("output.bsii", &bsii_binary).unwrap();
    let bsii_file = BsiiFile::parse(&bsii_binary).unwrap();
    println!("{}", bsii_file.to_siin());
}
