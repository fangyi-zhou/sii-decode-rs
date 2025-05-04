use std::env;
use std::fs;

use bsii_decode::BsiiFile;
use scsc_decrypt::ScscFile;
use simple_logger::SimpleLogger;

mod bsii_decode;
mod bsii_output;
mod scsc_decrypt;

fn main() {
    SimpleLogger::new().init().unwrap();

    let arg = env::args().last().unwrap();
    let content = fs::read(arg).unwrap();
    let scsc_file = ScscFile::from_content(content.as_slice()).unwrap();
    let bsii_binary = scsc_file.to_bsii_binary();
    // fs::write("output.bsii", &bsii_binary).unwrap();
    let bsii_file = BsiiFile::from_content(&bsii_binary).unwrap();
    println!("{}", bsii_output::bsii_to_siin(&bsii_file));
}
