use std::env;
use std::fs;

use sii_decode::bsii_file::BsiiFile;
use sii_decode::file_type::decode_to_bsii;
use sii_decode::save_data::{SaveDataExt, AchievementTracker, CargoCategory};

fn main() {
    let args: Vec<String> = env::args().collect();
    let path = if args.len() > 1 {
        &args[1]
    } else {
        "saves/my_save.sii"
    };

    let content = fs::read(path).expect("Failed to read file");
    let decoded = decode_to_bsii(&content).expect("Failed to decode file");

    if decoded.len() < 4 || &decoded[0..4] != b"BSII" {
        eprintln!("Decoded content is not BSII format");
        return;
    }

    let bsii = BsiiFile::parse(&decoded).expect("Failed to parse BSII");
    let entries = bsii.get_delivery_log_entries().expect("Failed to extract delivery log entries");

    // Use the integrated AchievementTracker
    let tracker = AchievementTracker::from_entries(&entries);
    let exp_beats_all = &tracker.experience_beats_all;

    println!("=== Integrated Achievement Progress: Experience Beats All! ===");
    for category in CargoCategory::all() {
        let count = exp_beats_all.category_counts.get(category).unwrap_or(&0);
        let status = if count > &0 { "[X]" } else { "[ ]" };
        println!("{} {:16} ({} deliveries)", status, category.name(), count);
    }

    println!("\nTotal categories completed: {}/8", exp_beats_all.completed_categories.len());
    if exp_beats_all.is_complete() {
        println!("Status: ACHIEVEMENT COMPLETED! 🎉");
    } else {
        println!("Status: In progress ({:.1}%)", exp_beats_all.progress() * 100.0);
    }

    // Also show brand distance while we are at it
    println!("\n=== Integrated Achievement Progress: 5 Truck Brands ===");
    let brand_ach = &tracker.brand_distance;
    for (brand, stats) in brand_ach.brands_by_distance() {
        let status = if stats.distance_km >= brand_ach.required_distance_km { "✓" } else { " " };
        println!("  {} {:12} {:>6} km", status, brand.name(), stats.distance_km);
    }
    println!("Qualifying brands: {}/{}", brand_ach.qualifying_brand_count(), brand_ach.required_brand_count);
}
