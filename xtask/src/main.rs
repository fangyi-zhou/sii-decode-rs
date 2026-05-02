use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const RUST_OUTPUT: &str = "src/ets2/generated/cargo_metadata.rs";
const JSON_OUTPUT: &str = "target/ets2-defs/cargo_metadata.json";

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
struct CargoMetadata {
    id: String,
    name: String,
    groups: Vec<String>,
    body_types: Vec<String>,
    trailer_categories: Vec<String>,
}

#[derive(Debug)]
enum Error {
    Usage(String),
    Io { path: PathBuf, source: io::Error },
    Parse { path: PathBuf, message: String },
}

impl Error {
    fn io(path: impl Into<PathBuf>, source: io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }

    fn parse(path: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Self::Parse {
            path: path.into(),
            message: message.into(),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Usage(message) => write!(f, "{message}"),
            Error::Io { path, source } => write!(f, "{}: {}", path.display(), source),
            Error::Parse { path, message } => write!(f, "{}: {}", path.display(), message),
        }
    }
}

impl std::error::Error for Error {}

fn main() {
    if let Err(err) = run(env::args().skip(1)) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn run(args: impl IntoIterator<Item = String>) -> Result<(), Error> {
    let mut args = args.into_iter();
    let command = args
        .next()
        .ok_or_else(|| Error::Usage(usage().to_string()))?;

    match command.as_str() {
        "extract-ets2-defs" => {
            let defs = parse_defs_arg(args)?;
            let cargo_metadata = extract_cargo_metadata(&defs)?;
            write_generated_files(&cargo_metadata)?;
            println!(
                "extracted {} cargo definitions to {RUST_OUTPUT}",
                cargo_metadata.len()
            );
            Ok(())
        }
        _ => Err(Error::Usage(usage().to_string())),
    }
}

fn usage() -> &'static str {
    "usage: cargo xtask extract-ets2-defs --defs def/def"
}

fn parse_defs_arg(mut args: impl Iterator<Item = String>) -> Result<PathBuf, Error> {
    let flag = args
        .next()
        .ok_or_else(|| Error::Usage(usage().to_string()))?;
    if flag != "--defs" {
        return Err(Error::Usage(usage().to_string()));
    }
    let defs = args
        .next()
        .ok_or_else(|| Error::Usage(usage().to_string()))?;
    if args.next().is_some() {
        return Err(Error::Usage(usage().to_string()));
    }
    Ok(PathBuf::from(defs))
}

fn extract_cargo_metadata(defs: &Path) -> Result<Vec<CargoMetadata>, Error> {
    let body_type_categories =
        parse_body_type_categories(&defs.join("body_type_to_trailer_category.sii"))?;
    let cargo_dir = defs.join("cargo");
    let mut cargo_paths = fs::read_dir(&cargo_dir)
        .map_err(|source| Error::io(&cargo_dir, source))?
        .map(|entry| {
            entry
                .map(|entry| entry.path())
                .map_err(|source| Error::io(&cargo_dir, source))
        })
        .collect::<Result<Vec<_>, _>>()?;

    cargo_paths.retain(|path| path.extension() == Some(OsStr::new("sui")));
    cargo_paths.sort();

    let mut cargos = Vec::new();
    for path in cargo_paths {
        let content = fs::read_to_string(&path).map_err(|source| Error::io(&path, source))?;
        cargos.push(parse_cargo_file(&path, &content, &body_type_categories)?);
    }
    cargos.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(cargos)
}

fn parse_cargo_file(
    path: &Path,
    content: &str,
    body_type_categories: &BTreeMap<String, String>,
) -> Result<CargoMetadata, Error> {
    let mut id = None;
    let mut name = None;
    let mut groups = BTreeSet::new();
    let mut body_types = BTreeSet::new();

    for line in normalized_lines(content) {
        if let Some(value) = line.strip_prefix("cargo_data:") {
            id = Some(parse_cargo_id(value.trim()).ok_or_else(|| {
                Error::parse(path, format!("invalid cargo_data declaration `{}`", line))
            })?);
        } else if let Some(value) = line.strip_prefix("name:") {
            name = Some(parse_value(value.trim()).to_string());
        } else if let Some(value) = line.strip_prefix("group[]:") {
            groups.insert(parse_value(value.trim()).to_string());
        } else if let Some(value) = line.strip_prefix("body_types[]:") {
            body_types.insert(parse_value(value.trim()).to_string());
        }
    }

    let id = id.ok_or_else(|| Error::parse(path, "missing cargo_data declaration"))?;
    let name = name.ok_or_else(|| Error::parse(path, "missing cargo name"))?;
    let body_types = body_types.into_iter().collect::<Vec<_>>();
    let trailer_categories = body_types
        .iter()
        .filter_map(|body_type| body_type_categories.get(body_type).cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();

    Ok(CargoMetadata {
        id,
        name,
        groups: groups.into_iter().collect(),
        body_types,
        trailer_categories,
    })
}

fn parse_body_type_categories(path: &Path) -> Result<BTreeMap<String, String>, Error> {
    let content = fs::read_to_string(path).map_err(|source| Error::io(path, source))?;
    let mut body_type_categories = BTreeMap::new();
    let mut current_category = None::<String>;

    for line in normalized_lines(&content) {
        if line.starts_with("trailer_category_def:") {
            current_category = None;
        } else if line == "}" {
            current_category = None;
        } else if let Some(value) = line.strip_prefix("trailer_category:") {
            current_category = Some(parse_value(value.trim()).to_string());
        } else if let Some(value) = line.strip_prefix("trailer_body_matches[]:") {
            let category = current_category.as_ref().ok_or_else(|| {
                Error::parse(path, "trailer_body_matches[] before trailer_category")
            })?;
            body_type_categories.insert(parse_value(value.trim()).to_string(), category.clone());
        }
    }

    Ok(body_type_categories)
}

fn normalized_lines(content: &str) -> impl Iterator<Item = String> + '_ {
    content.lines().filter_map(|line| {
        let line = strip_comment(line).trim();
        if line.is_empty() {
            None
        } else {
            Some(line.to_string())
        }
    })
}

fn strip_comment(line: &str) -> &str {
    let mut in_string = false;
    let mut escaped = false;

    for (index, ch) in line.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        match ch {
            '\\' if in_string => escaped = true,
            '"' => in_string = !in_string,
            '#' if !in_string => return &line[..index],
            _ => {}
        }
    }
    line
}

fn parse_cargo_id(value: &str) -> Option<String> {
    value
        .strip_prefix("cargo.")
        .filter(|id| !id.is_empty())
        .map(ToString::to_string)
}

fn parse_value(value: &str) -> &str {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(value)
}

fn write_generated_files(cargos: &[CargoMetadata]) -> Result<(), Error> {
    let rust_output = Path::new(RUST_OUTPUT);
    if let Some(parent) = rust_output.parent() {
        fs::create_dir_all(parent).map_err(|source| Error::io(parent, source))?;
    }
    fs::write(rust_output, render_rust(cargos)).map_err(|source| Error::io(rust_output, source))?;

    let json_output = Path::new(JSON_OUTPUT);
    if let Some(parent) = json_output.parent() {
        fs::create_dir_all(parent).map_err(|source| Error::io(parent, source))?;
    }
    fs::write(json_output, render_json(cargos)).map_err(|source| Error::io(json_output, source))?;
    Ok(())
}

fn render_rust(cargos: &[CargoMetadata]) -> String {
    let mut output = String::from(
        "// @generated by `cargo xtask extract-ets2-defs --defs def/def`\n\
         // Do not edit by hand.\n\n\
         use crate::ets2::CargoMetadata;\n\n\
         pub const CARGOS: &[CargoMetadata] = &[\n",
    );

    for cargo in cargos {
        output.push_str("    CargoMetadata {\n");
        output.push_str(&format!("        id: \"{}\",\n", rust_escape(&cargo.id)));
        output.push_str(&format!(
            "        name: \"{}\",\n",
            rust_escape(&cargo.name)
        ));
        output.push_str(&format!(
            "        groups: {},\n",
            render_rust_str_slice(&cargo.groups)
        ));
        output.push_str(&format!(
            "        body_types: {},\n",
            render_rust_str_slice(&cargo.body_types)
        ));
        output.push_str(&format!(
            "        trailer_categories: {},\n",
            render_rust_str_slice(&cargo.trailer_categories)
        ));
        output.push_str("    },\n");
    }

    output.push_str("];\n");
    output
}

fn render_rust_str_slice(values: &[String]) -> String {
    let values = values
        .iter()
        .map(|value| format!("\"{}\"", rust_escape(value)))
        .collect::<Vec<_>>()
        .join(", ");
    format!("&[{values}]")
}

fn rust_escape(value: &str) -> String {
    value.escape_default().to_string()
}

fn render_json(cargos: &[CargoMetadata]) -> String {
    let mut output = String::from("{\n  \"cargos\": [\n");
    for (index, cargo) in cargos.iter().enumerate() {
        if index > 0 {
            output.push_str(",\n");
        }
        output.push_str("    {\n");
        output.push_str(&format!("      \"id\": \"{}\",\n", json_escape(&cargo.id)));
        output.push_str(&format!(
            "      \"name\": \"{}\",\n",
            json_escape(&cargo.name)
        ));
        output.push_str(&format!(
            "      \"groups\": {},\n",
            render_json_array(&cargo.groups)
        ));
        output.push_str(&format!(
            "      \"body_types\": {},\n",
            render_json_array(&cargo.body_types)
        ));
        output.push_str(&format!(
            "      \"trailer_categories\": {}\n",
            render_json_array(&cargo.trailer_categories)
        ));
        output.push_str("    }");
    }
    output.push_str("\n  ]\n}\n");
    output
}

fn render_json_array(values: &[String]) -> String {
    let values = values
        .iter()
        .map(|value| format!("\"{}\"", json_escape(value)))
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{values}]")
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
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn parses_inline_cargo_snippet() {
        let categories = BTreeMap::from([
            ("curtainside".to_string(), "dryvan".to_string()),
            ("dryvan".to_string(), "dryvan".to_string()),
        ]);
        let cargo = parse_cargo_file(
            Path::new("def/def/cargo/apples.sui"),
            r#"
            cargo_data: cargo.apples
            {
                name: "@@cn_apples@@"
                group[]: refrigerated
                group[]: containers
                body_types[]: curtainside
                body_types[]: dryvan # inline comment
            }
            "#,
            &categories,
        )
        .unwrap();

        assert_eq!(cargo.id, "apples");
        assert_eq!(cargo.name, "@@cn_apples@@");
        assert_eq!(cargo.groups, ["containers", "refrigerated"]);
        assert_eq!(cargo.body_types, ["curtainside", "dryvan"]);
        assert_eq!(cargo.trailer_categories, ["dryvan"]);
    }

    #[test]
    fn parses_body_type_category_snippet() {
        let dir = temp_dir("body-type-category");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("body_type_to_trailer_category.sii");
        fs::write(
            &path,
            r#"
            SiiNunit
            {
            trailer_category_def: .category.1
            {
                trailer_category: tr_tank # token "tank" is taken
                trailer_body_matches[]: chemtank
                trailer_body_matches[]: foodtank
            }
            }
            "#,
        )
        .unwrap();

        let categories = parse_body_type_categories(&path).unwrap();
        fs::remove_dir_all(&dir).unwrap();

        assert_eq!(categories.get("chemtank").unwrap(), "tr_tank");
        assert_eq!(categories.get("foodtank").unwrap(), "tr_tank");
    }

    #[test]
    fn extracts_synthetic_def_tree_with_deterministic_ordering() {
        let defs = temp_dir("extract-defs").join("def");
        let cargo_dir = defs.join("cargo");
        fs::create_dir_all(&cargo_dir).unwrap();
        fs::write(
            defs.join("body_type_to_trailer_category.sii"),
            r#"
            trailer_category_def: .category.1
            {
                trailer_category: dryvan
                trailer_body_matches[]: dryvan
                trailer_body_matches[]: curtainside
            }
            "#,
        )
        .unwrap();
        fs::write(
            cargo_dir.join("z_gravel.sui"),
            r#"
            cargo_data: cargo.z_gravel
            {
                name: "@@cn_gravel@@"
                group[]: bulk
                body_types[]: dryvan
            }
            "#,
        )
        .unwrap();
        fs::write(
            cargo_dir.join("a_apples.sui"),
            r#"
            cargo_data: cargo.a_apples
            {
                name: "@@cn_apples@@"
                group[]: refrigerated
                body_types[]: curtainside
            }
            "#,
        )
        .unwrap();

        let cargos = extract_cargo_metadata(&defs).unwrap();
        let rust = render_rust(&cargos);
        fs::remove_dir_all(defs.parent().unwrap()).unwrap();

        assert_eq!(
            cargos
                .iter()
                .map(|cargo| cargo.id.as_str())
                .collect::<Vec<_>>(),
            ["a_apples", "z_gravel"]
        );
        assert!(rust.find("id: \"a_apples\"").unwrap() < rust.find("id: \"z_gravel\"").unwrap());
    }

    fn temp_dir(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        env::temp_dir().join(format!(
            "sii-decode-rs-{name}-{}-{nanos}",
            std::process::id()
        ))
    }
}
