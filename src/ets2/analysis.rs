use crate::bsii_file::BsiiFile;
use crate::ets2::{
    evaluate_achievements, Achievement, AchievementEvidence, AchievementRegistry,
    AchievementStatus, DeliveryAnalytics, SaveGame,
};
use crate::file_type::decode_until_bsii;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum AnalyzeError {
    Decode(String),
    BsiiParse(String),
    SaveGame(String),
}

impl std::fmt::Display for AnalyzeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnalyzeError::Decode(message) => write!(f, "decode error: {message}"),
            AnalyzeError::BsiiParse(message) => write!(f, "BSII parse error: {message}"),
            AnalyzeError::SaveGame(message) => write!(f, "save game error: {message}"),
        }
    }
}

impl std::error::Error for AnalyzeError {}

pub fn analyze_save_to_json(input: &[u8]) -> Result<String, AnalyzeError> {
    let bsii_content =
        decode_until_bsii(input).map_err(|err| AnalyzeError::Decode(err.to_string()))?;
    let bsii = BsiiFile::parse(bsii_content.as_ref())
        .map_err(|err| AnalyzeError::BsiiParse(err.to_string()))?;
    let save = SaveGame::from_bsii(&bsii).map_err(|err| AnalyzeError::SaveGame(err.to_string()))?;
    Ok(render_analysis_json(&save))
}

fn render_analysis_json(save: &SaveGame) -> String {
    let analytics = save.analytics();
    let registry = evaluate_achievements(save);
    let mut output = String::from("{\n");
    output.push_str("  \"analytics\": ");
    output.push_str(&render_analytics(&analytics));
    output.push_str(",\n  \"achievements\": ");
    output.push_str(&render_achievement_registry(&registry));
    output.push_str("\n}\n");
    output
}

fn render_analytics(analytics: &DeliveryAnalytics) -> String {
    let mut output = String::from("{\n");
    output.push_str(&format!(
        "    \"delivery_count\": {},\n",
        analytics.delivery_count
    ));
    output.push_str(&format!(
        "    \"total_distance_km\": {},\n",
        analytics.total_distance_km
    ));
    output.push_str(&format!(
        "    \"total_revenue\": {},\n",
        render_number(analytics.total_revenue)
    ));
    output.push_str(&format!(
        "    \"unique_cargos\": {},\n",
        render_string_iter(analytics.unique_cargos.iter().map(String::as_str))
    ));
    output.push_str(&format!(
        "    \"unique_companies\": {},\n",
        render_string_iter(analytics.unique_companies.iter().map(String::as_str))
    ));
    output.push_str(&format!(
        "    \"job_type_breakdown\": {},\n",
        render_usize_map(&analytics.job_type_breakdown)
    ));
    output.push_str(&format!(
        "    \"brand_distance_km\": {},\n",
        render_u64_map(&analytics.brand_distance_km)
    ));
    output.push_str(&format!(
        "    \"cargo_category_coverage\": {}\n",
        render_usize_map(&analytics.cargo_category_coverage)
    ));
    output.push_str("  }");
    output
}

fn render_achievement_registry(registry: &AchievementRegistry) -> String {
    let achievements = registry
        .achievements
        .iter()
        .map(render_achievement)
        .collect::<Vec<_>>()
        .join(",\n");
    format!("[\n{achievements}\n  ]")
}

fn render_achievement(achievement: &Achievement) -> String {
    format!(
        "    {{\n      \"id\": \"{}\",\n      \"display_name\": \"{}\",\n      \"description\": \"{}\",\n      \"status\": \"{}\",\n      \"progress\": {{ \"current\": {}, \"target\": {}, \"unit\": \"{}\" }},\n      \"evidence\": {}\n    }}",
        json_escape(achievement.id),
        json_escape(achievement.display_name),
        json_escape(achievement.description),
        render_status(achievement.status),
        achievement.progress.current,
        achievement.progress.target,
        json_escape(achievement.progress.unit),
        render_evidence(&achievement.evidence)
    )
}

fn render_status(status: AchievementStatus) -> &'static str {
    match status {
        AchievementStatus::Complete => "complete",
        AchievementStatus::InProgress => "in_progress",
    }
}

fn render_evidence(evidence: &[AchievementEvidence]) -> String {
    let evidence = evidence
        .iter()
        .map(|item| {
            format!(
                "{{ \"label\": \"{}\", \"value\": \"{}\", \"complete\": {} }}",
                json_escape(&item.label),
                json_escape(&item.value),
                item.complete
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{evidence}]")
}

fn render_string_iter<'a>(values: impl Iterator<Item = &'a str>) -> String {
    let values = values
        .map(|value| format!("\"{}\"", json_escape(value)))
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{values}]")
}

fn render_usize_map(map: &std::collections::BTreeMap<String, usize>) -> String {
    let fields = map
        .iter()
        .map(|(key, value)| format!("\"{}\": {}", json_escape(key), value))
        .collect::<Vec<_>>()
        .join(", ");
    format!("{{{fields}}}")
}

fn render_u64_map(map: &std::collections::BTreeMap<String, u64>) -> String {
    let fields = map
        .iter()
        .map(|(key, value)| format!("\"{}\": {}", json_escape(key), value))
        .collect::<Vec<_>>()
        .join(", ");
    format!("{{{fields}}}")
}

fn render_number(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{value:.1}")
    } else {
        value.to_string()
    }
}

fn json_escape(value: &str) -> String {
    let mut escaped = String::new();
    for ch in value.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            ch if ch.is_control() => escaped.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => escaped.push(ch),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ets2::{DeliveryLog, DeliveryLogEntry};

    #[test]
    fn renders_analysis_json() {
        let save = SaveGame {
            delivery_log: DeliveryLog::from_entries(vec![DeliveryLogEntry {
                source_company: "company.volatile.lkwlog.amsterdam".to_string(),
                destination_company: "company.volatile.stokes.amsterdam".to_string(),
                cargo: "cargo.gravel".to_string(),
                distance_km: 362,
                revenue: 16930.0,
                truck: "vehicle.mercedes.actros".to_string(),
                job_type: "cargo".to_string(),
            }]),
        };

        let json = render_analysis_json(&save);

        assert!(json.contains("\"delivery_count\": 1"));
        assert!(json.contains("\"total_distance_km\": 362"));
        assert!(json.contains("\"id\": \"experience_beats_all\""));
        assert!(json.contains("\"id\": \"test_drive_limited\""));
    }
}
