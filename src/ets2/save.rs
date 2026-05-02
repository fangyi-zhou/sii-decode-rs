use std::collections::{BTreeMap, BTreeSet};

use crate::bsii_file::{BsiiFile, DataBlock, DataValue};
use crate::ets2::generated::cargo_metadata::CARGOS;
use crate::ets2::CargoMetadata;

pub const COMPANY_PREFIX: &str = "company.volatile.";
pub const CARGO_PREFIX: &str = "cargo.";
pub const VEHICLE_PREFIX: &str = "vehicle.";

// Delivery log params are reverse engineered by the community:
// https://forum.scssoft.com/viewtopic.php?start=10&t=317674
const SOURCE_COMPANY_PARAM: usize = 1;
const DESTINATION_COMPANY_PARAM: usize = 2;
const CARGO_PARAM: usize = 3;
const REVENUE_PARAM: usize = 5;
const DISTANCE_PARAM: usize = 6;
const TRUCK_PARAM: usize = 16;
const JOB_TYPE_PARAM: usize = 18;
const MIN_PARAMS_LEN: usize = JOB_TYPE_PARAM + 1;

#[derive(Debug, Clone, PartialEq)]
pub struct SaveGame {
    pub delivery_log: DeliveryLog,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeliveryLog {
    pub entries: Vec<DeliveryLogEntry>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeliveryLogEntry {
    pub source_company: String,
    pub destination_company: String,
    pub cargo: String,
    pub distance_km: u32,
    pub revenue: f64,
    pub truck: String,
    pub job_type: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeliveryAnalytics {
    pub delivery_count: usize,
    pub total_distance_km: u64,
    pub total_revenue: f64,
    pub unique_cargos: BTreeSet<String>,
    pub unique_companies: BTreeSet<String>,
    pub job_type_breakdown: BTreeMap<String, usize>,
    pub brand_distance_km: BTreeMap<String, u64>,
    pub cargo_category_coverage: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SaveGameError {
    MissingDeliveryLog,
    MissingDeliveryLogEntries,
}

impl std::fmt::Display for SaveGameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SaveGameError::MissingDeliveryLog => write!(f, "missing delivery_log block"),
            SaveGameError::MissingDeliveryLogEntries => {
                write!(f, "missing delivery_log entries field")
            }
        }
    }
}

impl std::error::Error for SaveGameError {}

impl SaveGame {
    pub fn from_bsii(bsii: &BsiiFile<'_>) -> Result<Self, SaveGameError> {
        let delivery_log = DeliveryLog::from_bsii(bsii)?;
        Ok(Self { delivery_log })
    }

    pub fn analytics(&self) -> DeliveryAnalytics {
        self.delivery_log.analytics()
    }
}

impl DeliveryLog {
    pub fn from_entries(entries: Vec<DeliveryLogEntry>) -> Self {
        Self { entries }
    }

    pub fn from_bsii(bsii: &BsiiFile<'_>) -> Result<Self, SaveGameError> {
        let entry_blocks = ordered_delivery_entry_blocks(bsii)?;
        let entries = entry_blocks
            .into_iter()
            .filter_map(|block| DeliveryLogEntry::from_block(bsii, block))
            .collect();
        Ok(Self { entries })
    }

    pub fn delivery_count(&self) -> usize {
        self.entries.len()
    }

    pub fn total_distance_km(&self) -> u64 {
        self.entries
            .iter()
            .map(|entry| u64::from(entry.distance_km))
            .sum()
    }

    pub fn total_revenue(&self) -> f64 {
        self.entries.iter().map(|entry| entry.revenue).sum()
    }

    pub fn unique_cargos(&self) -> BTreeSet<String> {
        self.entries
            .iter()
            .map(|entry| cargo_id(&entry.cargo).to_string())
            .collect()
    }

    pub fn unique_companies(&self) -> BTreeSet<String> {
        self.entries
            .iter()
            .flat_map(|entry| {
                [
                    company_id(&entry.source_company).to_string(),
                    company_id(&entry.destination_company).to_string(),
                ]
            })
            .collect()
    }

    pub fn job_type_breakdown(&self) -> BTreeMap<String, usize> {
        count_by(self.entries.iter().map(|entry| entry.job_type.as_str()))
    }

    pub fn brand_distance_km(&self) -> BTreeMap<String, u64> {
        let mut distance_by_brand = BTreeMap::new();
        for entry in &self.entries {
            *distance_by_brand
                .entry(truck_brand(&entry.truck).to_string())
                .or_insert(0) += u64::from(entry.distance_km);
        }
        distance_by_brand
    }

    pub fn cargo_category_coverage(&self) -> BTreeMap<String, usize> {
        let metadata_by_cargo = cargo_metadata_by_id();
        let mut coverage = BTreeMap::new();
        for entry in &self.entries {
            if let Some(metadata) = metadata_by_cargo.get(cargo_id(&entry.cargo)) {
                for category in metadata.trailer_categories {
                    *coverage.entry((*category).to_string()).or_insert(0) += 1;
                }
            }
        }
        coverage
    }

    pub fn analytics(&self) -> DeliveryAnalytics {
        DeliveryAnalytics {
            delivery_count: self.delivery_count(),
            total_distance_km: self.total_distance_km(),
            total_revenue: self.total_revenue(),
            unique_cargos: self.unique_cargos(),
            unique_companies: self.unique_companies(),
            job_type_breakdown: self.job_type_breakdown(),
            brand_distance_km: self.brand_distance_km(),
            cargo_category_coverage: self.cargo_category_coverage(),
        }
    }
}

impl DeliveryLogEntry {
    pub fn from_params(params: &[&str]) -> Option<Self> {
        if params.len() < MIN_PARAMS_LEN {
            return None;
        }
        Some(Self {
            source_company: params[SOURCE_COMPANY_PARAM].to_string(),
            destination_company: params[DESTINATION_COMPANY_PARAM].to_string(),
            cargo: params[CARGO_PARAM].to_string(),
            distance_km: params[DISTANCE_PARAM].parse().ok()?,
            revenue: params[REVENUE_PARAM].parse().ok()?,
            truck: params[TRUCK_PARAM].to_string(),
            job_type: params[JOB_TYPE_PARAM].to_string(),
        })
    }

    fn from_block(bsii: &BsiiFile<'_>, block: &DataBlock<'_>) -> Option<Self> {
        let params = match block.field(bsii, "params")? {
            DataValue::StringArray(params) => params.as_slice(),
            _ => return None,
        };
        Self::from_params(params)
    }
}

fn ordered_delivery_entry_blocks<'a>(
    bsii: &'a BsiiFile<'a>,
) -> Result<Vec<&'a DataBlock<'a>>, SaveGameError> {
    let log_block = bsii
        .blocks_by_prototype_name("delivery_log")
        .next()
        .ok_or(SaveGameError::MissingDeliveryLog)?;
    let entries = match log_block.field(bsii, "entries") {
        Some(DataValue::IdArray(entries)) => entries,
        _ => return Err(SaveGameError::MissingDeliveryLogEntries),
    };

    let entry_blocks = entries
        .iter()
        .filter_map(|entry_id| {
            bsii.blocks_by_prototype_name("delivery_log_entry")
                .find(|block| &block.id == entry_id)
        })
        .collect();
    Ok(entry_blocks)
}

fn cargo_metadata_by_id() -> BTreeMap<&'static str, &'static CargoMetadata> {
    CARGOS.iter().map(|cargo| (cargo.id, cargo)).collect()
}

fn count_by<'a>(values: impl Iterator<Item = &'a str>) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for value in values {
        *counts.entry(value.to_string()).or_insert(0) += 1;
    }
    counts
}

pub fn cargo_id(cargo: &str) -> &str {
    cargo.strip_prefix(CARGO_PREFIX).unwrap_or(cargo)
}

pub fn company_id(company: &str) -> &str {
    company
        .strip_prefix(COMPANY_PREFIX)
        .unwrap_or(company)
        .split('.')
        .next()
        .unwrap_or(company)
}

pub fn truck_brand(truck: &str) -> &str {
    truck
        .strip_prefix(VEHICLE_PREFIX)
        .unwrap_or(truck)
        .split('.')
        .next()
        .unwrap_or(truck)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fs;
    use std::path::Path;

    use super::*;
    use crate::bsii_file::{DataBlock, Id, Prototype, ValuePrototype};
    use crate::file_type::decode_until_bsii;

    #[test]
    fn extracts_delivery_log_entries_from_bsii() {
        let first_entry_id = Id::Nameless(10);
        let second_entry_id = Id::Nameless(11);
        let bsii = synthetic_bsii(vec![first_entry_id.clone(), second_entry_id.clone()]);

        let save = SaveGame::from_bsii(&bsii).unwrap();

        assert_eq!(save.delivery_log.entries.len(), 2);
        assert_eq!(save.delivery_log.entries[0].cargo, "cargo.gravel");
        assert_eq!(save.delivery_log.entries[0].distance_km, 362);
        assert_eq!(save.delivery_log.entries[0].revenue, 16930.0);
        assert_eq!(save.delivery_log.entries[1].truck, "vehicle.scania.r");
    }

    #[test]
    fn computes_delivery_analytics() {
        let log = DeliveryLog::from_entries(vec![
            entry(
                "company.volatile.lkwlog.amsterdam",
                "company.volatile.stokes.amsterdam",
                "cargo.gravel",
                362,
                16930.0,
                "vehicle.mercedes.actros",
                "quick",
            ),
            entry(
                "company.volatile.lkwlog.amsterdam",
                "company.volatile.transinet.rotterdam",
                "cargo.canned_beef",
                1482,
                4830.5,
                "vehicle.scania.r",
                "cargo",
            ),
        ]);

        let analytics = log.analytics();

        assert_eq!(analytics.delivery_count, 2);
        assert_eq!(analytics.total_distance_km, 1844);
        assert_eq!(analytics.total_revenue, 21760.5);
        assert_eq!(analytics.unique_cargos, btreeset(["canned_beef", "gravel"]));
        assert_eq!(
            analytics.unique_companies,
            btreeset(["lkwlog", "stokes", "transinet"])
        );
        assert_eq!(
            analytics.job_type_breakdown,
            btreemap([("cargo", 1), ("quick", 1)])
        );
        assert_eq!(
            analytics.brand_distance_km,
            btreemap([("mercedes", 362_u64), ("scania", 1482)])
        );
        assert_eq!(
            analytics.cargo_category_coverage,
            btreemap([("bulk", 1), ("refrigerated", 1)])
        );
    }

    #[test]
    fn parses_delivery_params_and_ids() {
        let params = [
            "605",
            "company.volatile.lkwlog.amsterdam",
            "company.volatile.stokes.amsterdam",
            "cargo.gravel",
            "16",
            "16930.000",
            "362",
            "0.000",
            "295",
            "0",
            "0",
            "1",
            "1",
            "16930",
            "0",
            "600",
            "vehicle.mercedes.actros",
            "362",
            "quick",
            "",
            "",
            "0",
            "25000.000",
        ];
        let entry = DeliveryLogEntry::from_params(&params).unwrap();

        assert_eq!(entry.source_company, "company.volatile.lkwlog.amsterdam");
        assert_eq!(company_id(&entry.source_company), "lkwlog");
        assert_eq!(cargo_id(&entry.cargo), "gravel");
        assert_eq!(entry.distance_km, 362);
        assert_eq!(entry.revenue, 16930.0);
        assert_eq!(truck_brand(&entry.truck), "mercedes");
        assert_eq!(entry.job_type, "quick");
    }

    #[test]
    #[ignore = "requires ignored local save at saves/my_save.sii"]
    fn extracts_ignored_local_save() {
        let path = Path::new("saves/my_save.sii");
        if !path.exists() {
            return;
        }

        let content = fs::read(path).unwrap();
        let bsii_content = decode_until_bsii(&content).unwrap();
        let bsii = BsiiFile::parse(bsii_content.as_ref()).unwrap();
        let save = SaveGame::from_bsii(&bsii).unwrap();

        assert!(save.delivery_log.delivery_count() > 0);
    }

    fn synthetic_bsii(entry_ids: Vec<Id>) -> BsiiFile<'static> {
        let entries = entry_ids
            .iter()
            .enumerate()
            .map(|(index, id)| DataBlock {
                prototype_id: 2,
                id: id.clone(),
                data: vec![DataValue::StringArray(match index {
                    0 => params(
                        "cargo.gravel",
                        "vehicle.mercedes.actros",
                        "quick",
                        "362",
                        "16930.000",
                    ),
                    _ => params(
                        "cargo.beef_meat",
                        "vehicle.scania.r",
                        "cargo",
                        "1482",
                        "4830.500",
                    ),
                })],
            })
            .collect::<Vec<_>>();

        let mut data_blocks = vec![DataBlock {
            prototype_id: 1,
            id: Id::Nameless(1),
            data: vec![DataValue::IdArray(entry_ids)],
        }];
        data_blocks.extend(entries);

        BsiiFile {
            header: b"BSII",
            version: 2,
            prototypes: HashMap::from([
                (
                    1,
                    Prototype {
                        id: 1,
                        name: "delivery_log",
                        value_prototypes: vec![ValuePrototype {
                            type_id: 0x3a,
                            name: "entries",
                            enum_values: None,
                        }],
                    },
                ),
                (
                    2,
                    Prototype {
                        id: 2,
                        name: "delivery_log_entry",
                        value_prototypes: vec![ValuePrototype {
                            type_id: 0x02,
                            name: "params",
                            enum_values: None,
                        }],
                    },
                ),
            ]),
            data_blocks,
        }
    }

    fn params(
        cargo: &'static str,
        truck: &'static str,
        job_type: &'static str,
        distance: &'static str,
        revenue: &'static str,
    ) -> Vec<&'static str> {
        vec![
            "605",
            "company.volatile.lkwlog.amsterdam",
            "company.volatile.stokes.amsterdam",
            cargo,
            "16",
            revenue,
            distance,
            "0.000",
            "295",
            "0",
            "0",
            "1",
            "1",
            revenue,
            "0",
            "600",
            truck,
            distance,
            job_type,
            "",
            "",
            "0",
            "25000.000",
        ]
    }

    fn entry(
        source_company: &str,
        destination_company: &str,
        cargo: &str,
        distance_km: u32,
        revenue: f64,
        truck: &str,
        job_type: &str,
    ) -> DeliveryLogEntry {
        DeliveryLogEntry {
            source_company: source_company.to_string(),
            destination_company: destination_company.to_string(),
            cargo: cargo.to_string(),
            distance_km,
            revenue,
            truck: truck.to_string(),
            job_type: job_type.to_string(),
        }
    }

    fn btreeset<const N: usize>(values: [&str; N]) -> BTreeSet<String> {
        values.into_iter().map(ToString::to_string).collect()
    }

    fn btreemap<K, V, const N: usize>(values: [(K, V); N]) -> BTreeMap<String, V>
    where
        K: ToString,
    {
        values
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect()
    }
}
