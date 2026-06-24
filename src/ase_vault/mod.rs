//! Àṣẹ Vault — ese Ifá corpus loader
//!
//! Loads the 16 principal Odù JSON files at compile time via `include_str!`
//! and exposes them through a lazily-initialised static `HashMap`.
//!
//! # Lookup keys
//! Every Odù is registered under two keys so callers can use either form:
//! - full filename stem: `"00_eji_ogbe"`, `"01_oyeku_meji"`, …
//! - short name (no numeric prefix): `"eji_ogbe"`, `"oyeku_meji"`, …
//!
//! # Returned data
//! The JSON files have no top-level `ese` array; the verse material lives in:
//! - `metadata.ese_myth`   — a single narrative prose string
//! - `metadata.prescriptions` — an array of actionable verse lines
//!
//! `get_ese(odu_name)` returns the `prescriptions` array (Vec<String>).
//! `get_ese_myth(odu_name)` returns the `ese_myth` narrative string.

use serde_json::Value;
use std::collections::HashMap;
use std::sync::OnceLock;

// ── Static vault ─────────────────────────────────────────────────────────────

static VAULT: OnceLock<HashMap<String, Value>> = OnceLock::new();

/// Returns a reference to the lazily-initialised vault.
fn vault() -> &'static HashMap<String, Value> {
    VAULT.get_or_init(|| {
        let mut map: HashMap<String, Value> = HashMap::new();

        // Each tuple: (full_stem, short_name, json_content)
        let files: &[(&str, &str, &str)] = &[
            (
                "00_eji_ogbe",
                "eji_ogbe",
                include_str!("../../data/16_principals/00_eji_ogbe.json"),
            ),
            (
                "01_oyeku_meji",
                "oyeku_meji",
                include_str!("../../data/16_principals/01_oyeku_meji.json"),
            ),
            (
                "02_iwori_meji",
                "iwori_meji",
                include_str!("../../data/16_principals/02_iwori_meji.json"),
            ),
            (
                "03_odi_meji",
                "odi_meji",
                include_str!("../../data/16_principals/03_odi_meji.json"),
            ),
            (
                "04_irosun_meji",
                "irosun_meji",
                include_str!("../../data/16_principals/04_irosun_meji.json"),
            ),
            (
                "05_owonrin_meji",
                "owonrin_meji",
                include_str!("../../data/16_principals/05_owonrin_meji.json"),
            ),
            (
                "06_obara_meji",
                "obara_meji",
                include_str!("../../data/16_principals/06_obara_meji.json"),
            ),
            (
                "07_okanran_meji",
                "okanran_meji",
                include_str!("../../data/16_principals/07_okanran_meji.json"),
            ),
            (
                "08_ogunda_meji",
                "ogunda_meji",
                include_str!("../../data/16_principals/08_ogunda_meji.json"),
            ),
            (
                "09_osa_meji",
                "osa_meji",
                include_str!("../../data/16_principals/09_osa_meji.json"),
            ),
            (
                "10_ika_meji",
                "ika_meji",
                include_str!("../../data/16_principals/10_ika_meji.json"),
            ),
            (
                "11_oturupon_meji",
                "oturupon_meji",
                include_str!("../../data/16_principals/11_oturupon_meji.json"),
            ),
            (
                "12_otura_meji",
                "otura_meji",
                include_str!("../../data/16_principals/12_otura_meji.json"),
            ),
            (
                "13_irete_meji",
                "irete_meji",
                include_str!("../../data/16_principals/13_irete_meji.json"),
            ),
            (
                "14_ose_meji",
                "ose_meji",
                include_str!("../../data/16_principals/14_ose_meji.json"),
            ),
            (
                "15_ofun_meji",
                "ofun_meji",
                include_str!("../../data/16_principals/15_ofun_meji.json"),
            ),
        ];

        for (full_stem, short_name, content) in files {
            match serde_json::from_str::<Value>(content) {
                Ok(v) => {
                    map.insert(full_stem.to_string(), v.clone());
                    map.insert(short_name.to_string(), v);
                }
                Err(e) => {
                    // In a no-std / embedded context this would be a panic;
                    // here we log and continue so a single bad file can't
                    // silently break the whole vault.
                    eprintln!("[ase_vault] failed to parse {full_stem}: {e}");
                }
            }
        }

        map
    })
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Return the `prescriptions` array for the named Odù as a `Vec<String>`.
///
/// `odu_name` may be either the full stem (`"00_eji_ogbe"`) or the short name
/// (`"eji_ogbe"`).
///
/// Returns `None` if the name is not recognised or the JSON is malformed.
pub fn get_ese(odu_name: &str) -> Option<Vec<String>> {
    let entry = vault().get(odu_name)?;
    let prescriptions = entry
        .get("metadata")?
        .get("prescriptions")?
        .as_array()?;
    Some(
        prescriptions
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect(),
    )
}

/// Return the narrative `ese_myth` prose string for the named Odù.
///
/// `odu_name` accepts the same forms as [`get_ese`].
pub fn get_ese_myth(odu_name: &str) -> Option<String> {
    let entry = vault().get(odu_name)?;
    entry
        .get("metadata")?
        .get("ese_myth")?
        .as_str()
        .map(String::from)
}

/// Convenience lookup by the `odu_id` field (0–15) stored in the JSON.
///
/// Maps the numeric id to its canonical short name and delegates to
/// [`get_ese`].  Returns `None` for ids outside the 0–15 range.
pub fn get_ese_by_id(odu_id: u8) -> Option<Vec<String>> {
    let short_names: [&str; 16] = [
        "eji_ogbe",
        "oyeku_meji",
        "iwori_meji",
        "odi_meji",
        "irosun_meji",
        "owonrin_meji",
        "obara_meji",
        "okanran_meji",
        "ogunda_meji",
        "osa_meji",
        "ika_meji",
        "oturupon_meji",
        "otura_meji",
        "irete_meji",
        "ose_meji",
        "ofun_meji",
    ];
    let name = short_names.get(odu_id as usize)?;
    get_ese(name)
}

/// Return the full parsed JSON [`Value`] for the named Odù, if present.
///
/// Useful when callers need fields beyond `ese` and `ese_myth` (e.g.
/// `orisha_vector`, `hermetic_gates`, `governance`).
pub fn get_raw(odu_name: &str) -> Option<&'static Value> {
    vault().get(odu_name)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eji_ogbe_prescriptions_non_empty() {
        let ese = get_ese("eji_ogbe").expect("eji_ogbe should be in vault");
        assert!(!ese.is_empty(), "prescriptions should have at least one entry");
        // Spot-check the first prescription contains expected content
        assert!(
            ese[0].contains("clarity") || ese[0].len() > 10,
            "first prescription should be non-trivial"
        );
    }

    #[test]
    fn full_stem_lookup_works() {
        let a = get_ese("eji_ogbe");
        let b = get_ese("00_eji_ogbe");
        assert_eq!(a, b, "short and full-stem keys should return the same data");
    }

    #[test]
    fn ofun_meji_myth_non_empty() {
        let myth = get_ese_myth("ofun_meji").expect("ofun_meji myth should be present");
        assert!(!myth.is_empty());
    }

    #[test]
    fn get_ese_by_id_round_trips() {
        // id 0 == eji_ogbe
        let by_id = get_ese_by_id(0).expect("id 0 should resolve");
        let by_name = get_ese("eji_ogbe").expect("eji_ogbe should resolve");
        assert_eq!(by_id, by_name);
        // id 15 == ofun_meji
        assert!(get_ese_by_id(15).is_some());
        // id 16 is out of range
        assert!(get_ese_by_id(16).is_none());
    }

    #[test]
    fn all_16_principals_present() {
        let names = [
            "eji_ogbe", "oyeku_meji", "iwori_meji", "odi_meji",
            "irosun_meji", "owonrin_meji", "obara_meji", "okanran_meji",
            "ogunda_meji", "osa_meji", "ika_meji", "oturupon_meji",
            "otura_meji", "irete_meji", "ose_meji", "ofun_meji",
        ];
        for name in &names {
            assert!(
                get_ese(name).is_some(),
                "Missing prescriptions for: {name}"
            );
            assert!(
                get_ese_myth(name).is_some(),
                "Missing ese_myth for: {name}"
            );
        }
    }

    #[test]
    fn raw_value_has_orisha_vector() {
        let raw = get_raw("eji_ogbe").expect("raw should exist");
        assert!(
            raw.get("orisha_vector").is_some(),
            "JSON should contain orisha_vector"
        );
    }
}
