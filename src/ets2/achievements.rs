use std::collections::{BTreeMap, BTreeSet};

use crate::ets2::generated::cargo_metadata::CARGOS;
use crate::ets2::save::{cargo_id, company_id, truck_brand, DeliveryLogEntry, SaveGame};
use crate::ets2::CargoMetadata;

pub const EXPERIENCE_BEATS_ALL_CATEGORIES: [&str; 8] = [
    "Machinery",
    "ADR cargo",
    "Container",
    "Refrigerated",
    "Liquid cargo",
    "Fragile cargo",
    "Construction",
    "Bulk cargo",
];

const TEST_DRIVE_LIMITED_TARGET_BRANDS: u32 = 5;
const TEST_DRIVE_LIMITED_TARGET_DISTANCE_KM: u64 = 999;
const OWNED_TRUCK_JOB_TYPES: [&str; 4] = ["cargo", "external", "compn", "on_compn"];
const ALL_IS_POSSIBLE_TARGET_CARGOS: u32 = 30;
const RELIABLE_CONTRACTOR_TARGET_COMPANIES: u32 = 15;
const LONG_HAULER_TARGET_DISTANCE_KM: u32 = 2000;
const PROFIT_HUNTER_TARGET_REVENUE: f64 = 130_000.0;
const PROFIT_HUNTER_TARGET_DISTANCE_KM: u32 = 2200;
const SUMMARY_EVIDENCE_LIMIT: usize = 30;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AchievementRegistry {
    pub achievements: Vec<Achievement>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Achievement {
    pub id: &'static str,
    pub display_name: &'static str,
    pub description: &'static str,
    pub status: AchievementStatus,
    pub progress: AchievementProgress,
    pub evidence: Vec<AchievementEvidence>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum AchievementStatus {
    Complete,
    InProgress,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AchievementProgress {
    pub current: u32,
    pub target: u32,
    pub unit: &'static str,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AchievementEvidence {
    pub label: String,
    pub value: String,
    pub complete: bool,
}

pub fn evaluate_achievements(save: &SaveGame) -> AchievementRegistry {
    AchievementRegistry {
        achievements: vec![
            experience_beats_all(save),
            test_drive_limited(save),
            all_is_possible(save),
            reliable_contractor(save),
            long_hauler(save),
            profit_hunter(save),
        ],
    }
}

fn experience_beats_all(save: &SaveGame) -> Achievement {
    let mut evidence_by_category = EXPERIENCE_BEATS_ALL_CATEGORIES
        .into_iter()
        .map(|category| (category, BTreeSet::new()))
        .collect::<BTreeMap<_, _>>();

    for entry in &save.delivery_log.entries {
        for category in achievement_categories_for_cargo(&entry.cargo) {
            if let Some(cargos) = evidence_by_category.get_mut(category) {
                cargos.insert(cargo_id(&entry.cargo).to_string());
            }
        }
    }

    let evidence = EXPERIENCE_BEATS_ALL_CATEGORIES
        .into_iter()
        .map(|category| {
            let cargos = &evidence_by_category[category];
            AchievementEvidence {
                label: category.to_string(),
                value: if cargos.is_empty() {
                    "missing".to_string()
                } else {
                    cargos.iter().cloned().collect::<Vec<_>>().join(", ")
                },
                complete: !cargos.is_empty(),
            }
        })
        .collect::<Vec<_>>();
    let current = evidence.iter().filter(|item| item.complete).count() as u32;
    let target = EXPERIENCE_BEATS_ALL_CATEGORIES.len() as u32;

    Achievement {
        id: "experience_beats_all",
        display_name: "Experience Beats All",
        description: "Complete deliveries with all trailer types.",
        status: status_from_progress(current, target),
        progress: AchievementProgress {
            current,
            target,
            unit: "categories",
        },
        evidence,
    }
}

fn test_drive_limited(save: &SaveGame) -> Achievement {
    let mut distance_by_brand = BTreeMap::<String, u64>::new();
    for entry in save
        .delivery_log
        .entries
        .iter()
        .filter(|entry| is_owned_truck_job_type(&entry.job_type))
    {
        *distance_by_brand
            .entry(display_truck_brand(truck_brand(&entry.truck)).to_string())
            .or_insert(0) += u64::from(entry.distance_km);
    }

    let evidence = distance_by_brand
        .iter()
        .map(|(brand, distance)| AchievementEvidence {
            label: brand.clone(),
            value: format!("{distance} km"),
            complete: *distance >= TEST_DRIVE_LIMITED_TARGET_DISTANCE_KM,
        })
        .collect::<Vec<_>>();
    let current = evidence.iter().filter(|item| item.complete).count() as u32;

    Achievement {
        id: "test_drive_limited",
        display_name: "Test Drive Limited (5 Truck Brands)",
        description:
            "Drive at least 999 km during jobs with each of at least 5 owned-truck brands.",
        status: status_from_progress(current, TEST_DRIVE_LIMITED_TARGET_BRANDS),
        progress: AchievementProgress {
            current,
            target: TEST_DRIVE_LIMITED_TARGET_BRANDS,
            unit: "brands",
        },
        evidence,
    }
}

fn all_is_possible(save: &SaveGame) -> Achievement {
    let cargos = save
        .delivery_log
        .unique_cargos()
        .into_iter()
        .filter(|cargo| !cargo.is_empty())
        .collect::<BTreeSet<_>>();
    let current = cargos.len() as u32;

    Achievement {
        id: "all_is_possible",
        display_name: "All Is Possible",
        description: "Take and complete jobs with at least 30 different cargoes.",
        status: status_from_progress(current, ALL_IS_POSSIBLE_TARGET_CARGOS),
        progress: AchievementProgress {
            current,
            target: ALL_IS_POSSIBLE_TARGET_CARGOS,
            unit: "cargoes",
        },
        evidence: vec![AchievementEvidence {
            label: "Completed cargoes".to_string(),
            value: summarize_set(&cargos, SUMMARY_EVIDENCE_LIMIT),
            complete: current >= ALL_IS_POSSIBLE_TARGET_CARGOS,
        }],
    }
}

fn reliable_contractor(save: &SaveGame) -> Achievement {
    let companies = save
        .delivery_log
        .entries
        .iter()
        .flat_map(|entry| {
            [
                company_id(&entry.source_company).to_string(),
                company_id(&entry.destination_company).to_string(),
            ]
        })
        .filter(|company| !company.is_empty())
        .collect::<BTreeSet<_>>();
    let current = companies.len() as u32;

    Achievement {
        id: "reliable_contractor",
        display_name: "Reliable Contractor",
        description: "Perform jobs for at least 15 different companies in the game.",
        status: status_from_progress(current, RELIABLE_CONTRACTOR_TARGET_COMPANIES),
        progress: AchievementProgress {
            current,
            target: RELIABLE_CONTRACTOR_TARGET_COMPANIES,
            unit: "companies",
        },
        evidence: vec![AchievementEvidence {
            label: "Companies".to_string(),
            value: summarize_set(&companies, SUMMARY_EVIDENCE_LIMIT),
            complete: current >= RELIABLE_CONTRACTOR_TARGET_COMPANIES,
        }],
    }
}

fn long_hauler(save: &SaveGame) -> Achievement {
    let longest = save
        .delivery_log
        .entries
        .iter()
        .max_by_key(|entry| entry.distance_km);
    let current = longest.map_or(0, |entry| entry.distance_km);
    let complete = current > LONG_HAULER_TARGET_DISTANCE_KM;

    Achievement {
        id: "long_hauler",
        display_name: "Long Hauler",
        description: "Complete a delivery that was greater than 2,000 km.",
        status: if complete {
            AchievementStatus::Complete
        } else {
            AchievementStatus::InProgress
        },
        progress: AchievementProgress {
            current,
            target: LONG_HAULER_TARGET_DISTANCE_KM,
            unit: "km",
        },
        evidence: vec![AchievementEvidence {
            label: "Longest delivery".to_string(),
            value: longest
                .map(delivery_summary)
                .unwrap_or_else(|| "missing".to_string()),
            complete,
        }],
    }
}

fn profit_hunter(save: &SaveGame) -> Achievement {
    let qualifying = save
        .delivery_log
        .entries
        .iter()
        .filter(|entry| is_profit_hunter_delivery(entry))
        .max_by(|left, right| left.revenue.total_cmp(&right.revenue));
    let evidence_entry = qualifying.or_else(|| {
        save.delivery_log
            .entries
            .iter()
            .max_by(|left, right| left.revenue.total_cmp(&right.revenue))
    });
    let current = evidence_entry.map_or(0, profit_hunter_criteria_met);
    let complete = qualifying.is_some();

    Achievement {
        id: "profit_hunter",
        display_name: "Profit Hunter",
        description: "Complete one job worth over 130,000 EUR and at least 2,200 km.",
        status: if complete {
            AchievementStatus::Complete
        } else {
            AchievementStatus::InProgress
        },
        progress: AchievementProgress {
            current,
            target: 2,
            unit: "requirements",
        },
        evidence: vec![AchievementEvidence {
            label: if complete {
                "Qualifying job".to_string()
            } else {
                "Best revenue job".to_string()
            },
            value: evidence_entry
                .map(delivery_summary)
                .unwrap_or_else(|| "missing".to_string()),
            complete,
        }],
    }
}

fn status_from_progress(current: u32, target: u32) -> AchievementStatus {
    if current >= target {
        AchievementStatus::Complete
    } else {
        AchievementStatus::InProgress
    }
}

pub fn is_owned_truck_job_type(job_type: &str) -> bool {
    OWNED_TRUCK_JOB_TYPES.contains(&job_type)
}

pub fn achievement_categories_for_cargo(cargo: &str) -> BTreeSet<&'static str> {
    let Some(metadata) = CARGOS
        .iter()
        .find(|metadata| metadata.id == cargo_id(cargo))
    else {
        return BTreeSet::new();
    };
    achievement_categories_for_metadata(metadata)
}

fn achievement_categories_for_metadata(metadata: &CargoMetadata) -> BTreeSet<&'static str> {
    let mut categories = BTreeSet::new();
    if metadata.groups.contains(&"machinery") {
        categories.insert("Machinery");
    }
    if metadata.adr_class.is_some() || metadata.groups.contains(&"adr") {
        categories.insert("ADR cargo");
    }
    if metadata.groups.contains(&"containers") || metadata.trailer_categories.contains(&"container")
    {
        categories.insert("Container");
    }
    if metadata.groups.contains(&"refrigerated")
        || metadata.trailer_categories.contains(&"refrigerated")
    {
        categories.insert("Refrigerated");
    }
    if metadata.groups.contains(&"liquid") || metadata.trailer_categories.contains(&"tr_tank") {
        categories.insert("Liquid cargo");
    }
    if metadata.fragility.is_some()
        || metadata.id == "glass_packed"
        || metadata.groups.contains(&"inloader")
    {
        categories.insert("Fragile cargo");
    }
    if metadata.groups.contains(&"construction") {
        categories.insert("Construction");
    }
    if metadata.groups.contains(&"bulk") || metadata.trailer_categories.contains(&"bulk") {
        categories.insert("Bulk cargo");
    }
    categories
}

fn display_truck_brand(brand: &str) -> &str {
    match brand {
        "daf" => "DAF",
        "iveco" => "Iveco",
        "man" => "MAN",
        "mercedes" => "Mercedes-Benz",
        "renault" => "Renault",
        "scania" => "Scania",
        "volvo" => "Volvo",
        _ => brand,
    }
}

fn summarize_set(values: &BTreeSet<String>, limit: usize) -> String {
    if values.is_empty() {
        return "missing".to_string();
    }

    let mut summary = values.iter().take(limit).cloned().collect::<Vec<_>>();
    if values.len() > limit {
        summary.push(format!("+{} more", values.len() - limit));
    }
    summary.join(", ")
}

fn delivery_summary(entry: &DeliveryLogEntry) -> String {
    format!(
        "{}, {} km, {}",
        cargo_id(&entry.cargo),
        entry.distance_km,
        format_revenue(entry.revenue)
    )
}

fn format_revenue(revenue: f64) -> String {
    if revenue.fract() == 0.0 {
        format!("{revenue:.0} EUR")
    } else {
        format!("{revenue:.2} EUR")
    }
}

fn is_profit_hunter_delivery(entry: &DeliveryLogEntry) -> bool {
    entry.revenue > PROFIT_HUNTER_TARGET_REVENUE
        && entry.distance_km >= PROFIT_HUNTER_TARGET_DISTANCE_KM
}

fn profit_hunter_criteria_met(entry: &DeliveryLogEntry) -> u32 {
    u32::from(entry.revenue > PROFIT_HUNTER_TARGET_REVENUE)
        + u32::from(entry.distance_km >= PROFIT_HUNTER_TARGET_DISTANCE_KM)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ets2::{DeliveryLog, DeliveryLogEntry};

    #[test]
    fn evaluates_experience_beats_all() {
        let save = save_with_entries(vec![
            entry("cargo.digger1000", 200, "vehicle.man.tgx", "quick"),
            entry("cargo.hydrogen", 200, "vehicle.daf.xf", "quick"),
            entry("cargo.apples_c", 200, "vehicle.iveco.hiway", "quick"),
            entry("cargo.canned_beef", 200, "vehicle.mercedes.actros", "quick"),
            entry("cargo.acid", 200, "vehicle.renault.t", "quick"),
            entry("cargo.glass_packed", 200, "vehicle.scania.r", "quick"),
            entry("cargo.bricks", 200, "vehicle.volvo.fh16", "quick"),
            entry("cargo.gravel", 200, "vehicle.man.tgx", "quick"),
        ]);

        let achievement = evaluate_achievements(&save).achievements.remove(0);

        assert_eq!(achievement.id, "experience_beats_all");
        assert_eq!(achievement.status, AchievementStatus::Complete);
        assert_eq!(achievement.progress.current, 8);
        assert!(achievement
            .evidence
            .iter()
            .all(|evidence| evidence.complete));
    }

    #[test]
    fn evaluates_test_drive_limited_owned_truck_jobs_only() {
        let save = save_with_entries(vec![
            entry("cargo.gravel", 999, "vehicle.man.tgx", "cargo"),
            entry("cargo.gravel", 1000, "vehicle.daf.xf", "external"),
            entry("cargo.gravel", 1001, "vehicle.mercedes.actros", "compn"),
            entry("cargo.gravel", 1002, "vehicle.renault.t", "on_compn"),
            entry("cargo.gravel", 1003, "vehicle.scania.r", "cargo"),
            entry("cargo.gravel", 5000, "vehicle.volvo.fh16", "quick"),
            entry("cargo.gravel", 5000, "vehicle.iveco.hiway", "freerm"),
            entry("cargo.gravel", 5000, "vehicle.iveco.hiway", "spec_oversize"),
            entry("cargo.gravel", 5000, "vehicle.iveco.hiway", "unknown"),
        ]);

        let achievement = evaluate_achievements(&save).achievements.remove(1);

        assert_eq!(achievement.id, "test_drive_limited");
        assert_eq!(
            achievement.display_name,
            "Test Drive Limited (5 Truck Brands)"
        );
        assert_eq!(achievement.status, AchievementStatus::Complete);
        assert_eq!(achievement.progress.current, 5);
        assert!(achievement
            .evidence
            .iter()
            .any(|evidence| evidence.label == "Mercedes-Benz" && evidence.complete));
        assert!(!achievement
            .evidence
            .iter()
            .any(|evidence| evidence.label == "Volvo"));
    }

    #[test]
    fn evaluates_all_is_possible() {
        let mut entries = (0..30)
            .map(|index| {
                entry(
                    &format!("cargo.cargo_{index}"),
                    200,
                    "vehicle.man.tgx",
                    "quick",
                )
            })
            .collect::<Vec<_>>();
        entries.push(entry("", 200, "vehicle.man.tgx", "quick"));
        let save = save_with_entries(entries);

        let achievement = achievement_by_id(&save, "all_is_possible");

        assert_eq!(achievement.status, AchievementStatus::Complete);
        assert_eq!(achievement.progress.current, 30);
        assert_eq!(achievement.progress.target, 30);
        assert!(achievement.evidence[0].complete);
    }

    #[test]
    fn evaluates_reliable_contractor() {
        let mut entries = (0..8)
            .map(|index| DeliveryLogEntry {
                source_company: format!("company.volatile.source_{index}.amsterdam"),
                destination_company: format!("company.volatile.destination_{index}.rotterdam"),
                ..entry("cargo.gravel", 200, "vehicle.man.tgx", "quick")
            })
            .collect::<Vec<_>>();
        entries.push(DeliveryLogEntry {
            source_company: "".to_string(),
            destination_company: "company.volatile.source_0.amsterdam".to_string(),
            ..entry("cargo.gravel", 200, "vehicle.man.tgx", "quick")
        });
        let save = save_with_entries(entries);

        let achievement = achievement_by_id(&save, "reliable_contractor");

        assert_eq!(achievement.status, AchievementStatus::Complete);
        assert_eq!(achievement.progress.current, 16);
        assert_eq!(achievement.progress.target, 15);
        assert!(achievement.evidence[0].value.contains("source_0"));
    }

    #[test]
    fn evaluates_long_hauler() {
        let save = save_with_entries(vec![
            entry("cargo.gravel", 2000, "vehicle.man.tgx", "quick"),
            entry("cargo.canned_beef", 2001, "vehicle.scania.r", "quick"),
        ]);

        let achievement = achievement_by_id(&save, "long_hauler");

        assert_eq!(achievement.status, AchievementStatus::Complete);
        assert_eq!(achievement.progress.current, 2001);
        assert_eq!(achievement.progress.target, 2000);
        assert_eq!(
            achievement.evidence[0].value,
            "canned_beef, 2001 km, 1000 EUR"
        );
    }

    #[test]
    fn evaluates_profit_hunter_as_single_qualifying_job() {
        let save = save_with_entries(vec![
            revenue_entry("cargo.gravel", 2300, 130_001.0),
            revenue_entry("cargo.canned_beef", 3000, 100_000.0),
            revenue_entry("cargo.bricks", 1000, 180_000.0),
        ]);

        let achievement = achievement_by_id(&save, "profit_hunter");

        assert_eq!(achievement.status, AchievementStatus::Complete);
        assert_eq!(achievement.progress.current, 2);
        assert_eq!(achievement.evidence[0].label, "Qualifying job");
        assert_eq!(achievement.evidence[0].value, "gravel, 2300 km, 130001 EUR");
    }

    #[test]
    fn profit_hunter_stays_in_progress_when_requirements_are_split_across_jobs() {
        let save = save_with_entries(vec![
            revenue_entry("cargo.canned_beef", 3000, 100_000.0),
            revenue_entry("cargo.bricks", 1000, 180_000.0),
        ]);

        let achievement = achievement_by_id(&save, "profit_hunter");

        assert_eq!(achievement.status, AchievementStatus::InProgress);
        assert_eq!(achievement.progress.current, 1);
        assert_eq!(achievement.evidence[0].label, "Best revenue job");
    }

    #[test]
    fn owned_truck_job_type_filter_matches_v1_rules() {
        for job_type in ["cargo", "external", "compn", "on_compn"] {
            assert!(is_owned_truck_job_type(job_type));
        }
        for job_type in ["quick", "freerm", "spec_oversize", "unknown", ""] {
            assert!(!is_owned_truck_job_type(job_type));
        }
    }

    #[test]
    fn derives_achievement_categories_from_cargo_metadata() {
        assert!(achievement_categories_for_cargo("cargo.digger1000").contains("Machinery"));
        assert!(achievement_categories_for_cargo("cargo.hydrogen").contains("ADR cargo"));
        assert!(achievement_categories_for_cargo("cargo.acid").contains("ADR cargo"));
        assert!(achievement_categories_for_cargo("cargo.apples_c").contains("Container"));
        assert!(achievement_categories_for_cargo("cargo.canned_beef").contains("Refrigerated"));
        assert!(achievement_categories_for_cargo("cargo.acid").contains("Liquid cargo"));
        assert!(achievement_categories_for_cargo("cargo.acid").contains("Fragile cargo"));
        assert!(achievement_categories_for_cargo("cargo.glass_packed").contains("Fragile cargo"));
        assert!(achievement_categories_for_cargo("cargo.bricks").contains("Construction"));
        assert!(achievement_categories_for_cargo("cargo.gravel").contains("Bulk cargo"));
    }

    fn save_with_entries(entries: Vec<DeliveryLogEntry>) -> SaveGame {
        SaveGame {
            delivery_log: DeliveryLog::from_entries(entries),
        }
    }

    fn entry(cargo: &str, distance_km: u32, truck: &str, job_type: &str) -> DeliveryLogEntry {
        DeliveryLogEntry {
            source_company: "company.volatile.lkwlog.amsterdam".to_string(),
            destination_company: "company.volatile.stokes.amsterdam".to_string(),
            cargo: cargo.to_string(),
            distance_km,
            revenue: 1000.0,
            truck: truck.to_string(),
            job_type: job_type.to_string(),
        }
    }

    fn revenue_entry(cargo: &str, distance_km: u32, revenue: f64) -> DeliveryLogEntry {
        DeliveryLogEntry {
            revenue,
            ..entry(cargo, distance_km, "vehicle.man.tgx", "quick")
        }
    }

    fn achievement_by_id(save: &SaveGame, id: &str) -> Achievement {
        evaluate_achievements(save)
            .achievements
            .into_iter()
            .find(|achievement| achievement.id == id)
            .unwrap()
    }
}
