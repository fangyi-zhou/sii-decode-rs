pub mod generated;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CargoMetadata {
    pub id: &'static str,
    pub name: &'static str,
    pub groups: &'static [&'static str],
    pub body_types: &'static [&'static str],
    pub trailer_categories: &'static [&'static str],
}
