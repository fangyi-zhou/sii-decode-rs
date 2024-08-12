use std::env;
use std::fs;

use scsc_decrypt::ScscFile;

mod scsc_decrypt;

fn main() {
    let arg = env::args().last().unwrap();
    let content = fs::read(arg).unwrap();
    let scsc_file = ScscFile::from_content(content.as_slice());
    let bsii_binary = scsc_file.to_bsii_binary();
    let header = &bsii_binary[0..4];
    println!("BSII header {:?}", header);
}
