//! Typed data structures for ETS2/ATS save files.
//!
//! This module provides strongly-typed representations of save file objects
//! that can be extracted from the generic `DataBlock` structures.

use std::collections::HashMap;

use crate::bsii_file::{BsiiFile, DataBlock, DataValue, Id};

/// Error type for save data extraction
#[derive(Debug)]
pub enum ExtractError {
    /// The data block has an unexpected prototype name
    WrongPrototype {
        expected: &'static str,
        found: String,
    },
    /// A required field is missing
    MissingField(&'static str),
    /// A field has an unexpected type
    WrongFieldType {
        field: &'static str,
        expected: &'static str,
    },
    /// The params array is too short
    ParamsTooShort { expected: usize, found: usize },
    /// A param has an unexpected type
    WrongParamType {
        index: usize,
        expected: &'static str,
    },
}

impl std::fmt::Display for ExtractError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExtractError::WrongPrototype { expected, found } => {
                write!(f, "Expected prototype '{}', found '{}'", expected, found)
            }
            ExtractError::MissingField(field) => write!(f, "Missing field '{}'", field),
            ExtractError::WrongFieldType { field, expected } => {
                write!(f, "Field '{}' has wrong type, expected {}", field, expected)
            }
            ExtractError::ParamsTooShort { expected, found } => {
                write!(
                    f,
                    "Params array too short: expected {}, found {}",
                    expected, found
                )
            }
            ExtractError::WrongParamType { index, expected } => {
                write!(
                    f,
                    "Param at index {} has wrong type, expected {}",
                    index, expected
                )
            }
        }
    }
}

impl std::error::Error for ExtractError {}

/// Job type for deliveries
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobType {
    /// Quick job (freight market)
    Quick,
    /// Cargo market job
    Cargo,
    /// External contracts
    External,
    /// World of Trucks job
    WorldOfTrucks,
    /// Special oversized cargo transport
    SpecialOversize,
    /// Free roam (damage/event tracking, not a real delivery)
    FreeRoam,
    /// Company job (owned trailers)
    Company,
    /// Online company job
    OnlineCompany,
    /// Unknown job type
    Unknown(String),
}

impl From<&str> for JobType {
    fn from(s: &str) -> Self {
        match s {
            "quick" => JobType::Quick,
            "cargo" => JobType::Cargo,
            "external" => JobType::External,
            "wot" => JobType::WorldOfTrucks,
            "spec_oversize" => JobType::SpecialOversize,
            "freerm" => JobType::FreeRoam,
            "compn" => JobType::Company,
            "on_compn" => JobType::OnlineCompany,
            _ => JobType::Unknown(s.to_string()),
        }
    }
}

/// A single delivery log entry representing a completed delivery.
///
/// The params array in the save file contains various delivery statistics.
/// Based on analysis of save files, the structure is:
/// - params[0]: Game time (minutes since game start)
/// - params[1]: Source company (e.g., "company.volatile.lkwlog.hamburg")
/// - params[2]: Destination company (e.g., "company.volatile.transinet.hamburg")
/// - params[3]: Cargo type (e.g., "cargo.sand")
/// - params[4]: Cargo mass in kg
/// - params[5]: Revenue earned
/// - params[6]: Unknown (possibly damage count)
/// - params[7]: Damage percentage as string (e.g., "0.000")
/// - params[8]: Distance in km
/// - params[9]: Unknown
/// - params[10]: Late delivery flag (0 = on time, 1 = late)
/// - params[11]: Unknown
/// - params[12]: Unknown
/// - params[13]: Base revenue (before bonuses)
/// - params[14]: Unknown
/// - params[15]: Time limit in minutes
/// - params[16]: Vehicle type (e.g., "vehicle.scania.r")
/// - params[17]: Cargo units count
/// - params[18]: Job type (quick, cargo, external, wot)
///
/// Version 1 (24 params) has additional fields:
/// - params[19-20]: Special flags (usually empty strings)
/// - params[21]: Unknown
/// - params[22]: Total weight as string
/// - params[23]: Cargo percentage (usually 100)
#[derive(Debug, Clone)]
pub struct DeliveryLogEntry {
    /// Unique identifier for this entry
    pub id: Id,
    /// Game time when delivery was completed (minutes since game start)
    pub game_time: i64,
    /// Source company identifier
    pub source_company: String,
    /// Destination company identifier
    pub destination_company: String,
    /// Cargo type identifier
    pub cargo: String,
    /// Cargo mass in kilograms
    pub cargo_mass_kg: i64,
    /// Revenue earned from this delivery
    pub revenue: i64,
    /// Cargo damage percentage (0.0 - 100.0)
    pub damage_percentage: f64,
    /// Distance traveled in kilometers
    pub distance_km: i64,
    /// Whether the delivery was late
    pub is_late: bool,
    /// Base revenue before bonuses
    pub base_revenue: i64,
    /// Time limit for delivery in minutes
    pub time_limit_minutes: i64,
    /// Vehicle type used for delivery
    pub vehicle: String,
    /// Number of cargo units
    pub cargo_units: i64,
    /// Type of job
    pub job_type: JobType,
    /// Total weight (if available, version 1 only)
    pub total_weight: Option<f64>,
    /// Cargo percentage (if available, version 1 only)
    pub cargo_percentage: Option<i64>,
}

impl DeliveryLogEntry {
    /// Extract source city from the company identifier.
    /// E.g., "company.volatile.lkwlog.hamburg" -> "hamburg"
    pub fn source_city(&self) -> Option<&str> {
        self.source_company.rsplit('.').next()
    }

    /// Extract destination city from the company identifier.
    /// E.g., "company.volatile.transinet.hamburg" -> "hamburg"
    pub fn destination_city(&self) -> Option<&str> {
        self.destination_company.rsplit('.').next()
    }

    /// Extract source company name from the identifier.
    /// E.g., "company.volatile.lkwlog.hamburg" -> "lkwlog"
    pub fn source_company_name(&self) -> Option<&str> {
        let parts: Vec<&str> = self.source_company.split('.').collect();
        if parts.len() >= 3 {
            Some(parts[parts.len() - 2])
        } else {
            None
        }
    }

    /// Extract destination company name from the identifier.
    /// E.g., "company.volatile.transinet.hamburg" -> "transinet"
    pub fn destination_company_name(&self) -> Option<&str> {
        let parts: Vec<&str> = self.destination_company.split('.').collect();
        if parts.len() >= 3 {
            Some(parts[parts.len() - 2])
        } else {
            None
        }
    }

    /// Extract cargo name from the cargo identifier.
    /// E.g., "cargo.sand" -> "sand"
    pub fn cargo_name(&self) -> Option<&str> {
        self.cargo.strip_prefix("cargo.")
    }

    /// Check if cargo was delivered without damage
    pub fn is_undamaged(&self) -> bool {
        self.damage_percentage == 0.0
    }
}

/// The delivery log containing all completed deliveries.
#[derive(Debug, Clone)]
pub struct DeliveryLog {
    /// Unique identifier for this log
    pub id: Id,
    /// Version of the delivery log format
    pub version: u32,
    /// List of delivery entry IDs
    pub entry_ids: Vec<Id>,
    /// Cached jobs count (may not be present in older versions)
    pub cached_jobs_count: Option<u32>,
}

/// A string parameter that may represent either a string or a numeric value.
/// In BSII format, the params array stores all values as strings.
#[derive(Debug, Clone)]
struct StringParam<'a>(&'a str);

impl<'a> StringParam<'a> {
    /// Try to parse as i64
    fn as_i64(&self) -> Option<i64> {
        self.0.parse().ok()
    }

    /// Get as string reference
    fn as_str(&self) -> &str {
        self.0
    }

    /// Try to parse as f64
    fn as_f64(&self) -> Option<f64> {
        self.0.parse().ok()
    }
}

impl DeliveryLogEntry {
    /// Create a DeliveryLogEntry from a DataBlock.
    ///
    /// The data block must be a "delivery_log_entry" prototype.
    pub fn from_data_block<'a>(
        data_block: &'a DataBlock<'a>,
        bsii_file: &'a BsiiFile<'a>,
    ) -> Result<Self, ExtractError> {
        let prototype = bsii_file
            .get_prototype(data_block.prototype_id)
            .ok_or(ExtractError::MissingField("prototype"))?;

        if prototype.name != "delivery_log_entry" {
            return Err(ExtractError::WrongPrototype {
                expected: "delivery_log_entry",
                found: prototype.name.to_string(),
            });
        }

        // Find the params field index
        let params_idx = prototype
            .value_prototypes
            .iter()
            .position(|vp| vp.name == "params")
            .ok_or(ExtractError::MissingField("params"))?;

        // The params array is stored as StringArray (type 0x02)
        // All values including numbers are stored as strings
        let params_data = &data_block.data[params_idx];

        let params: Vec<&str> = match params_data {
            DataValue::StringArray(arr) => arr.to_vec(),
            _ => {
                return Err(ExtractError::WrongFieldType {
                    field: "params",
                    expected: "StringArray",
                })
            }
        };

        let min_params = 19;
        if params.len() < min_params {
            return Err(ExtractError::ParamsTooShort {
                expected: min_params,
                found: params.len(),
            });
        }

        // Helper to get param as StringParam
        let p = |i: usize| StringParam(params[i]);

        // Extract values from params - all stored as strings
        let game_time = p(0).as_i64().ok_or(ExtractError::WrongParamType {
            index: 0,
            expected: "i64",
        })?;
        let source_company = p(1).as_str().to_string();
        let destination_company = p(2).as_str().to_string();
        let cargo = p(3).as_str().to_string();
        let cargo_mass_kg = p(4).as_i64().ok_or(ExtractError::WrongParamType {
            index: 4,
            expected: "i64",
        })?;
        let revenue = p(5).as_i64().ok_or(ExtractError::WrongParamType {
            index: 5,
            expected: "i64",
        })?;
        let damage_percentage = p(7).as_f64().unwrap_or(0.0);
        let distance_km = p(8).as_i64().ok_or(ExtractError::WrongParamType {
            index: 8,
            expected: "i64",
        })?;
        let is_late = p(10).as_i64().ok_or(ExtractError::WrongParamType {
            index: 10,
            expected: "i64",
        })? != 0;
        let base_revenue = p(13).as_i64().ok_or(ExtractError::WrongParamType {
            index: 13,
            expected: "i64",
        })?;
        let time_limit_minutes = p(15).as_i64().ok_or(ExtractError::WrongParamType {
            index: 15,
            expected: "i64",
        })?;
        let vehicle = p(16).as_str().to_string();
        let cargo_units = p(17).as_i64().ok_or(ExtractError::WrongParamType {
            index: 17,
            expected: "i64",
        })?;
        let job_type = JobType::from(p(18).as_str());

        // Version 1 (24 params) has additional fields
        let (total_weight, cargo_percentage) = if params.len() >= 24 {
            let weight = p(22).as_f64();
            let percentage = p(23).as_i64();
            (weight, percentage)
        } else {
            (None, None)
        };

        Ok(DeliveryLogEntry {
            id: data_block.id.clone(),
            game_time,
            source_company,
            destination_company,
            cargo,
            cargo_mass_kg,
            revenue,
            damage_percentage,
            distance_km,
            is_late,
            base_revenue,
            time_limit_minutes,
            vehicle,
            cargo_units,
            job_type,
            total_weight,
            cargo_percentage,
        })
    }
}

impl DeliveryLog {
    /// Create a DeliveryLog from a DataBlock.
    ///
    /// The data block must be a "delivery_log" prototype.
    pub fn from_data_block<'a>(
        data_block: &'a DataBlock<'a>,
        bsii_file: &'a BsiiFile<'a>,
    ) -> Result<Self, ExtractError> {
        let prototype = bsii_file
            .get_prototype(data_block.prototype_id)
            .ok_or(ExtractError::MissingField("prototype"))?;

        if prototype.name != "delivery_log" {
            return Err(ExtractError::WrongPrototype {
                expected: "delivery_log",
                found: prototype.name.to_string(),
            });
        }

        // Build a field name to index mapping
        let field_indices: HashMap<&str, usize> = prototype
            .value_prototypes
            .iter()
            .enumerate()
            .map(|(i, vp)| (vp.name, i))
            .collect();

        // Extract version
        let version_idx = field_indices
            .get("version")
            .ok_or(ExtractError::MissingField("version"))?;
        let version = match &data_block.data[*version_idx] {
            DataValue::UInt32(v) => *v,
            DataValue::Int32(v) => *v as u32,
            _ => {
                return Err(ExtractError::WrongFieldType {
                    field: "version",
                    expected: "u32",
                })
            }
        };

        // Extract entries (array of IDs)
        let entries_idx = field_indices
            .get("entries")
            .ok_or(ExtractError::MissingField("entries"))?;
        let entry_ids = match &data_block.data[*entries_idx] {
            DataValue::IdArray(ids) => ids.to_vec(),
            _ => {
                return Err(ExtractError::WrongFieldType {
                    field: "entries",
                    expected: "IdArray",
                })
            }
        };

        // Extract cached_jobs_count (optional - not present in older versions)
        let cached_jobs_count =
            field_indices
                .get("cached_jobs_count")
                .and_then(|idx| match &data_block.data[*idx] {
                    DataValue::UInt32(v) => Some(*v),
                    DataValue::Int32(v) => Some(*v as u32),
                    _ => None,
                });

        Ok(DeliveryLog {
            id: data_block.id.clone(),
            version,
            entry_ids,
            cached_jobs_count,
        })
    }
}

/// Extension trait for BsiiFile to extract typed save data
pub trait SaveDataExt {
    /// Find and extract the delivery log from the save file
    fn get_delivery_log(&self) -> Result<Option<DeliveryLog>, ExtractError>;

    /// Find and extract all delivery log entries from the save file
    fn get_delivery_log_entries(&self) -> Result<Vec<DeliveryLogEntry>, ExtractError>;

    /// Get all data blocks of a specific prototype name
    fn get_blocks_by_prototype(&self, name: &str) -> Vec<&DataBlock<'_>>;
}

impl SaveDataExt for BsiiFile<'_> {
    fn get_delivery_log(&self) -> Result<Option<DeliveryLog>, ExtractError> {
        for block in &self.data_blocks {
            if let Some(prototype) = self.get_prototype(block.prototype_id) {
                if prototype.name == "delivery_log" {
                    return DeliveryLog::from_data_block(block, self).map(Some);
                }
            }
        }
        Ok(None)
    }

    fn get_delivery_log_entries(&self) -> Result<Vec<DeliveryLogEntry>, ExtractError> {
        let mut entries = Vec::new();
        for block in &self.data_blocks {
            if let Some(prototype) = self.get_prototype(block.prototype_id) {
                if prototype.name == "delivery_log_entry" {
                    entries.push(DeliveryLogEntry::from_data_block(block, self)?);
                }
            }
        }
        Ok(entries)
    }

    fn get_blocks_by_prototype(&self, name: &str) -> Vec<&DataBlock<'_>> {
        self.data_blocks
            .iter()
            .filter(|block| {
                self.get_prototype(block.prototype_id)
                    .map(|p| p.name == name)
                    .unwrap_or(false)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_type_from_str() {
        assert_eq!(JobType::from("quick"), JobType::Quick);
        assert_eq!(JobType::from("cargo"), JobType::Cargo);
        assert_eq!(JobType::from("external"), JobType::External);
        assert_eq!(JobType::from("wot"), JobType::WorldOfTrucks);
        assert_eq!(JobType::from("spec_oversize"), JobType::SpecialOversize);
        assert_eq!(JobType::from("freerm"), JobType::FreeRoam);
        assert_eq!(JobType::from("compn"), JobType::Company);
        assert_eq!(JobType::from("on_compn"), JobType::OnlineCompany);
        assert!(matches!(JobType::from("unknown"), JobType::Unknown(_)));
    }

    #[test]
    fn test_delivery_log_entry_helpers() {
        let entry = DeliveryLogEntry {
            id: Id::Nameless(12345),
            game_time: 608,
            source_company: "company.volatile.lkwlog.hamburg".to_string(),
            destination_company: "company.volatile.transinet.frankfurt".to_string(),
            cargo: "cargo.sand".to_string(),
            cargo_mass_kg: 140500,
            revenue: 362,
            damage_percentage: 0.0,
            distance_km: 292,
            is_late: false,
            base_revenue: 362,
            time_limit_minutes: 600,
            vehicle: "vehicle.scania.r".to_string(),
            cargo_units: 1,
            job_type: JobType::Quick,
            total_weight: None,
            cargo_percentage: None,
        };

        assert_eq!(entry.source_city(), Some("hamburg"));
        assert_eq!(entry.destination_city(), Some("frankfurt"));
        assert_eq!(entry.source_company_name(), Some("lkwlog"));
        assert_eq!(entry.destination_company_name(), Some("transinet"));
        assert_eq!(entry.cargo_name(), Some("sand"));
        assert!(entry.is_undamaged());
    }

    // The following tests require external save files and are commented out for CI.
    // Uncomment locally to test with actual save files.

    // #[test]
    // fn test_inspect_delivery_log_entry_structure() {
    //     // This test helps us understand the actual structure of the params field
    //     let home = std::env::var("HOME").unwrap();
    //     // game_1.sii is encrypted (ScsC format), which will decode to BSII
    //     let path = format!("{}/Downloads/game_1.sii", home);
    //
    //     if !std::path::Path::new(&path).exists() {
    //         eprintln!("Skipping test: {} not found", path);
    //         return;
    //     }
    //
    //     let content = std::fs::read(&path).unwrap();
    //     let decoded = crate::file_type::decode_to_bsii(&content).unwrap();
    //
    //     // Check if we got BSII format
    //     if &decoded[0..4] != b"BSII" {
    //         eprintln!("Decoded content is not BSII format");
    //         return;
    //     }
    //
    //     let bsii = BsiiFile::parse(&decoded).unwrap();
    //
    //     // Find delivery_log_entry blocks and inspect their structure
    //     for block in &bsii.data_blocks {
    //         if let Some(proto) = bsii.get_prototype(block.prototype_id) {
    //             if proto.name == "delivery_log_entry" {
    //                 println!("\n=== delivery_log_entry structure ===");
    //                 println!("ID: {:?}", block.id);
    //
    //                 for (i, (data, vproto)) in
    //                     block.data.iter().zip(&proto.value_prototypes).enumerate()
    //                 {
    //                     println!(
    //                         "Field {}: name='{}', type_id=0x{:02x}, value={:?}",
    //                         i, vproto.name, vproto.type_id, data
    //                     );
    //                 }
    //
    //                 // Only inspect the first entry
    //                 break;
    //             }
    //         }
    //     }
    // }

    // #[test]
    // fn test_extract_delivery_log_entries() {
    //     let home = std::env::var("HOME").unwrap();
    //     let path = format!("{}/Downloads/game_1.sii", home);
    //
    //     if !std::path::Path::new(&path).exists() {
    //         eprintln!("Skipping test: {} not found", path);
    //         return;
    //     }
    //
    //     let content = std::fs::read(&path).unwrap();
    //     let decoded = crate::file_type::decode_to_bsii(&content).unwrap();
    //
    //     if &decoded[0..4] != b"BSII" {
    //         eprintln!("Decoded content is not BSII format");
    //         return;
    //     }
    //
    //     let bsii = BsiiFile::parse(&decoded).unwrap();
    //
    //     // Test extracting delivery log
    //     let delivery_log = bsii.get_delivery_log().unwrap();
    //     assert!(delivery_log.is_some(), "Should find delivery_log");
    //
    //     let log = delivery_log.unwrap();
    //     println!("Delivery Log version: {}", log.version);
    //     println!("Number of entries: {}", log.entry_ids.len());
    //     if let Some(count) = log.cached_jobs_count {
    //         println!("Cached jobs count: {}", count);
    //     }
    //
    //     // Test extracting delivery log entries
    //     let entries = bsii.get_delivery_log_entries().unwrap();
    //     println!("Extracted {} delivery log entries", entries.len());
    //
    //     for entry in &entries {
    //         println!("\n=== Delivery Entry ===");
    //         println!("  Game time: {} minutes", entry.game_time);
    //         println!(
    //             "  Source: {} ({})",
    //             entry.source_company_name().unwrap_or("?"),
    //             entry.source_city().unwrap_or("?")
    //         );
    //         println!(
    //             "  Destination: {} ({})",
    //             entry.destination_company_name().unwrap_or("?"),
    //             entry.destination_city().unwrap_or("?")
    //         );
    //         println!(
    //             "  Cargo: {} ({} kg)",
    //             entry.cargo_name().unwrap_or("?"),
    //             entry.cargo_mass_kg
    //         );
    //         println!("  Distance: {} km", entry.distance_km);
    //         println!("  Revenue: {}", entry.revenue);
    //         println!("  Damage: {}%", entry.damage_percentage);
    //         println!("  Late: {}", entry.is_late);
    //         println!("  Vehicle: {}", entry.vehicle);
    //         println!("  Job type: {:?}", entry.job_type);
    //     }
    //
    //     assert!(!entries.is_empty(), "Should have at least one entry");
    // }
}
