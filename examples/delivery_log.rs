use std::env;
use std::fs;

use sii_decode::bsii_file::BsiiFile;
use sii_decode::file_type::decode_to_bsii;
use sii_decode::save_data::SaveDataExt;

fn main() {
    let args: Vec<String> = env::args().collect();
    let path = if args.len() > 1 {
        &args[1]
    } else {
        "saves/my_save.sii"
    };

    println!("Reading save file: {}", path);

    let content = fs::read(path).expect("Failed to read file");
    let decoded = decode_to_bsii(&content).expect("Failed to decode file");

    // Check if we got BSII format
    if decoded.len() < 4 || &decoded[0..4] != b"BSII" {
        eprintln!("Decoded content is not BSII format (might be text SiiN format)");
        return;
    }

    let bsii = BsiiFile::parse(&decoded).expect("Failed to parse BSII");

    // Extract delivery log
    let delivery_log = bsii
        .get_delivery_log()
        .expect("Failed to extract delivery log");

    match delivery_log {
        Some(log) => {
            println!("\n=== Delivery Log ===");
            println!("Version: {}", log.version);
            println!("Total entries: {}", log.entry_ids.len());
            if let Some(count) = log.cached_jobs_count {
                println!("Cached jobs count: {}", count);
            }
        }
        None => {
            println!("No delivery log found in save file");
            return;
        }
    }

    // Extract delivery log entries
    let entries = bsii
        .get_delivery_log_entries()
        .expect("Failed to extract delivery log entries");

    println!("\n=== Delivery Entries ({}) ===", entries.len());

    for (i, entry) in entries.iter().enumerate() {
        println!("\n--- Delivery #{} ---", i + 1);
        println!(
            "  Route: {} ({}) -> {} ({})",
            entry.source_company_name().unwrap_or("-"),
            entry.source_city().unwrap_or("-"),
            entry.destination_company_name().unwrap_or("-"),
            entry.destination_city().unwrap_or("-")
        );
        println!(
            "  Cargo: {} ({} kg, {} units)",
            entry.cargo_name().unwrap_or("?"),
            entry.cargo_mass_kg,
            entry.cargo_units
        );
        println!("  Distance: {} km", entry.distance_km);
        println!(
            "  Revenue: {} (base: {})",
            entry.revenue, entry.base_revenue
        );
        println!("  Damage: {}%", entry.damage_percentage);
        println!(
            "  On time: {}",
            if entry.is_late { "No (late)" } else { "Yes" }
        );
        println!("  Vehicle: {}", entry.vehicle);
        println!("  Job type: {:?}", entry.job_type);
        println!(
            "  Time limit: {} minutes ({:.1} hours)",
            entry.time_limit_minutes,
            entry.time_limit_minutes as f64 / 60.0
        );
    }

    // Summary statistics
    if !entries.is_empty() {
        println!("\n=== Summary ===");
        let total_revenue: i64 = entries.iter().map(|e| e.revenue).sum();
        let total_distance: i64 = entries.iter().map(|e| e.distance_km).sum();
        let late_count = entries.iter().filter(|e| e.is_late).count();
        let undamaged_count = entries.iter().filter(|e| e.is_undamaged()).count();

        println!("Total revenue: {}", total_revenue);
        println!("Total distance: {} km", total_distance);
        println!(
            "On-time deliveries: {}/{}",
            entries.len() - late_count,
            entries.len()
        );
        println!(
            "Perfect deliveries (no damage): {}/{}",
            undamaged_count,
            entries.len()
        );
    }
}
