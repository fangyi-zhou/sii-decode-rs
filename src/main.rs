use std::env;
use std::fs;

use simple_logger::SimpleLogger;

fn main() {
    SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()
        .unwrap();

    let args = env::args().skip(1).collect::<Vec<_>>();
    let (analyze_ets2, path) = match args.as_slice() {
        [flag, path] if flag == "--ets2-achievements" => (true, path),
        [path] => (false, path),
        _ => panic!("usage: sii-decode [--ets2-achievements] path/to/file.sii"),
    };

    let content = fs::read(path).unwrap();
    if analyze_ets2 {
        println!(
            "{}",
            sii_decode::ets2::analyze_save_to_json(&content).unwrap()
        );
    } else {
        let decoded = sii_decode::file_type::decode_until_siin(&content).unwrap();
        println!("{}", String::from_utf8(decoded).unwrap());
    }
}
