mod achievements;
pub mod generated;
mod save;

pub use achievements::{
    achievement_categories_for_cargo, evaluate_achievements, is_owned_truck_job_type, Achievement,
    AchievementEvidence, AchievementProgress, AchievementRegistry, AchievementStatus,
    EXPERIENCE_BEATS_ALL_CATEGORIES,
};
pub use save::{
    DeliveryAnalytics, DeliveryLog, DeliveryLogEntry, SaveGame, SaveGameError, CARGO_PREFIX,
    COMPANY_PREFIX, VEHICLE_PREFIX,
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CargoMetadata {
    pub id: &'static str,
    pub name: &'static str,
    pub groups: &'static [&'static str],
    pub body_types: &'static [&'static str],
    pub trailer_categories: &'static [&'static str],
}
