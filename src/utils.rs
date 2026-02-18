use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};

lazy_static! {

    static ref WS: Regex = Regex::new(r#"\s+"#).unwrap();
    static ref DEFAULT_FILTERS: Vec<Regex> = vec![
        // Remove our discount prefix from the name before canonicalizing, since it doesn't affect the actual product
        r#"💲.*?% OFF💲"#,
        // Remove everything within brackets, including the brackets themselves
        r#"\[.*?\]"#,
        // Remove everything non alphanumeric except whitespace
        r#"[^a-zA-Z0-9!?,.\-\s]"#,
    ]
    .iter()
    .map(|s| Regex::new(s).unwrap())
    .collect::<Vec<_>>();
}

pub fn format_name<T: Into<String>>(name: T) -> String {
    let mut name = name.into().to_lowercase();
    name = name
        .replace(
            |c: char| !c.is_ascii_alphanumeric() && !c.is_whitespace(),
            "",
        )
        .trim()
        .to_string();
    name = name.replace(|c: char| c.is_whitespace(), "-");
    name
}

pub fn is_censored<T: Into<String>>(s: T) -> bool {
    let name = s.into();
    name.chars().all(|c| c == '#' || c.is_whitespace())
}

pub fn canonical_name<T: Into<String>>(s: T, filters: &Option<Vec<Regex>>) -> String {
    let mut out = s.into();

    let temp = Vec::new();
    let mut name_filters: &Vec<Regex> = filters.as_ref().unwrap_or(&temp);

    if name_filters.len() == 0 {
        name_filters = &DEFAULT_FILTERS;
    }

    for filter in name_filters {
        out = filter.replace_all(&out, " ").to_string();
    }

    out = WS.replace_all(&out, " ").trim().to_string();
    out
}

pub fn deserialize_regex_vec<'de, D>(deserializer: D) -> Result<Option<Vec<Regex>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt = Option::<Vec<String>>::deserialize(deserializer)?;

    match opt {
        Some(strings) => {
            let regexes = strings
                .into_iter()
                .map(|s| {
                    Regex::new(&s)
                        .map_err(|e| serde::de::Error::custom(format!("Invalid regex `{s}`: {e}")))
                })
                .collect::<Result<Vec<_>, _>>()?;

            Ok(Some(regexes))
        }
        None => Ok(None),
    }
}

pub fn serialize_regex_vec<S>(
    regexes: &Option<Vec<Regex>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match regexes {
        Some(regexes) => {
            let strings = regexes
                .iter()
                .map(|r| r.as_str().to_string())
                .collect::<Vec<_>>();
            strings.serialize(serializer)
        }
        None => serializer.serialize_none(),
    }
}
