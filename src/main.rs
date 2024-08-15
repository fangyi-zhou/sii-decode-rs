use std::env;
use std::fs;

use bsii_decode::BsiiFile;
use scsc_decrypt::ScscFile;
use simple_logger::SimpleLogger;

mod bsii_decode;
mod scsc_decrypt;

fn main() {
    SimpleLogger::new().init().unwrap();

    let arg = env::args().last().unwrap();
    let content = fs::read(arg).unwrap();
    let scsc_file = ScscFile::from_content(content.as_slice());
    let bsii_binary = scsc_file.to_bsii_binary();
    fs::write("output.bsii", &bsii_binary).unwrap();
    let _bsii_file = BsiiFile::from_content(&bsii_binary);
}
