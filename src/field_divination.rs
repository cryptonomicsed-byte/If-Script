//! field_divination — casting Odù against the Waggle field's real history.
//!
//! Connection Map v2 §3. Classical divination reads a fixed table; field
//! divination reads *what actually happened*: the diviner casts against the
//! journal (recall) and the live field (sniff), and the Odù emerges from the
//! operational record instead of a random throw.
//!
//! ## The Odù–channel mapping (principled, not hand-authored)
//!
//! An Odù is an 8-bit figure: two 4-bit tetragrams. Field divination derives
//! each tetragram from the four load-bearing channel readings of a territory
//! — the *signature* of a field state:
//!
//! ```text
//! bit 3 (1000): gold      — effective gold present  (fortune)
//! bit 2 (0100): bounded   — stability >= 0.5        (firm ground)
//! bit 1 (0010): taboo     — a live ethical exclusion (warning)
//! bit 0 (0001): dead-end  — failed paths outnumber help (loss)
//! ```
//!
//! The cast composes **present signature (top tetragram) over past
//! signature (bottom tetragram)** — exactly the structure of a classical
//! figure, where the right leg is what was and the left leg is what comes.
//! `0b1100_1100` (gold on firm ground, then and now) lands in the
//! Vision/Growth waves; `0b0011_1100` (was bright, now taboo over failure)
//! lands where the corpus keeps its warnings. The grounding is the field
//! math itself: each of the 256 figures corresponds to one (past, present)
//! pair of real channel states.
//!
//! All verbs fail soft into `CastError::FieldUnreachable` — divination
//! requires a field; it does not invent one.

use crate::odu::{get_odu_by_binary, Odu};
use serde_json::Value;
use std::time::Duration;

const DEFAULT_WAGGLE: &str = "http://127.0.0.1:7777";

#[derive(Debug, thiserror::Error)]
pub enum CastError {
    #[error("the field is unreachable (is waggled running at {0}?)")]
    FieldUnreachable(String),
    #[error("recall requires the substrate to run with a journal (-data)")]
    NoJournal,
}

/// One field-grounded cast: the figure, its corpus entry, and the readings
/// that produced it — the diviner shows the work.
pub struct FieldCast {
    pub odu: &'static Odu,
    pub binary: u8,
    pub present_signature: u8,
    pub past_signature: u8,
    pub present: ChannelReading,
    pub past: ChannelReading,
}

/// The four channel readings a tetragram is derived from.
#[derive(Debug, Default, Clone, Copy)]
pub struct ChannelReading {
    pub gold: f64,     // effective (trust-weighted, inhibition-applied)
    pub bounded: f64,  // stability 0..1
    pub taboo: f64,    // raw intensity 0..10
    pub dead_end: f64, // raw intensity 0..10
    pub help: f64,     // raw intensity 0..10
}

impl ChannelReading {
    /// The tetragram: see the module docs for the bit meanings.
    pub fn signature(&self) -> u8 {
        let mut sig = 0u8;
        if self.gold >= 1.0 {
            sig |= 0b1000;
        }
        if self.bounded >= 0.5 {
            sig |= 0b0100;
        }
        if self.taboo >= 1.0 {
            sig |= 0b0010;
        }
        if self.dead_end > self.help && self.dead_end >= 1.0 {
            sig |= 0b0001;
        }
        sig
    }
}

impl std::fmt::Debug for FieldCast {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FieldCast")
            .field("odu", &self.odu.name)
            .field("binary", &format_args!("{:#010b}", self.binary))
            .field(
                "present_signature",
                &format_args!("{:#06b}", self.present_signature),
            )
            .field(
                "past_signature",
                &format_args!("{:#06b}", self.past_signature),
            )
            .field("present", &self.present)
            .field("past", &self.past)
            .finish()
    }
}

pub struct FieldDiviner {
    http: reqwest::blocking::Client,
    base: String,
}

impl Default for FieldDiviner {
    fn default() -> Self {
        Self::new(std::env::var("WAGGLE_URL").unwrap_or_else(|_| DEFAULT_WAGGLE.to_string()))
    }
}

impl FieldDiviner {
    pub fn new(base: impl Into<String>) -> Self {
        Self {
            http: reqwest::blocking::Client::builder()
                .timeout(Duration::from_secs(5))
                .build()
                .expect("client"),
            base: base.into(),
        }
    }

    /// §3.1 `cast(uri_pattern)` — the full field cast: present state over
    /// the state `hours_back` ago (default one day), composed into a figure
    /// and matched to its Odù from real operational history.
    pub fn cast(&self, uri_pattern: &str) -> Result<FieldCast, CastError> {
        self.cast_at(uri_pattern, 24.0)
    }

    pub fn cast_at(&self, uri_pattern: &str, hours_back: f64) -> Result<FieldCast, CastError> {
        let present = self.read_now(uri_pattern)?;
        let past = self.read_recalled(uri_pattern, hours_back)?;
        let (ps, xs) = (present.signature(), past.signature());
        let binary = (ps << 4) | xs;
        Ok(FieldCast {
            odu: get_odu_by_binary(binary),
            binary,
            present_signature: ps,
            past_signature: xs,
            present,
            past,
        })
    }

    /// §3.2 `cast_bounded(uri_pattern)` — the stability oracle: divination
    /// over the Mandelbrot channel alone. Not "what happened here" but "is
    /// this a robust island or a fragile boundary". The figure is composed
    /// from stability quartiles now (top) and then (bottom), so rising
    /// ground and crumbling ground cast *different* Odù even at the same
    /// present stability.
    pub fn cast_bounded(&self, uri_pattern: &str) -> Result<FieldCast, CastError> {
        let s_now = self.bounded_stability(uri_pattern, None)?;
        let s_then = self.bounded_stability(uri_pattern, Some(24.0))?;
        // quartile tetragrams: 0..3 → 0b0000, 0b0001, 0b0011, 0b0111, 0b1111
        let quart = |s: f64| -> u8 {
            match (s * 4.0) as u8 {
                0 => 0b0000,
                1 => 0b0001,
                2 => 0b0011,
                3 => 0b0111,
                _ => 0b1111,
            }
        };
        let (ps, xs) = (quart(s_now), quart(s_then));
        let binary = (ps << 4) | xs;
        let present = ChannelReading {
            bounded: s_now,
            ..Default::default()
        };
        let past = ChannelReading {
            bounded: s_then,
            ..Default::default()
        };
        Ok(FieldCast {
            odu: get_odu_by_binary(binary),
            binary,
            present_signature: ps,
            past_signature: xs,
            present,
            past,
        })
    }

    /// §3.4 `cast_federated(remote_prefix, uri_pattern)` — divine across
    /// ecosystems: Vantage imports a remote field under a namespace prefix
    /// with the trust discount already applied at import, so the same cast
    /// against `<remote>/<pattern>` reads another deployment's history
    /// through the same interface, discounts included.
    pub fn cast_federated(
        &self,
        remote_prefix: &str,
        uri_pattern: &str,
    ) -> Result<FieldCast, CastError> {
        let namespaced = format!(
            "{}/{}",
            remote_prefix.trim_end_matches('/'),
            uri_pattern.trim_start_matches('/')
        );
        self.cast(&namespaced)
    }

    // ── field reads ─────────────────────────────────────────────────────

    fn get(&self, path: &str) -> Result<Value, CastError> {
        self.http
            .get(format!("{}{}", self.base, path))
            .send()
            .and_then(|r| r.json())
            .map_err(|_| CastError::FieldUnreachable(self.base.clone()))
    }

    fn read_now(&self, prefix: &str) -> Result<ChannelReading, CastError> {
        let out = self.get(&format!("/v1/sniff?prefix={prefix}&limit=200"))?;
        Ok(Self::fold(out.get("signals").and_then(Value::as_array)))
    }

    fn read_recalled(&self, prefix: &str, hours_back: f64) -> Result<ChannelReading, CastError> {
        let at = chrono::Utc::now() - chrono::Duration::seconds((hours_back * 3600.0) as i64);
        let out = self.get(&format!(
            "/v1/recall?prefix={prefix}&at={}&limit=200",
            at.format("%Y-%m-%dT%H:%M:%SZ")
        ))?;
        if out.get("error").is_some() {
            return Err(CastError::NoJournal);
        }
        Ok(Self::fold(out.get("signals").and_then(Value::as_array)))
    }

    fn bounded_stability(&self, prefix: &str, hours_back: Option<f64>) -> Result<f64, CastError> {
        let reading = match hours_back {
            None => self.read_now(prefix)?,
            Some(h) => self.read_recalled(prefix, h)?,
        };
        Ok(reading.bounded)
    }

    /// Fold raw signals into the four channel readings. Gold uses the
    /// effective intensity when the substrate provides it (tier weights and
    /// cross-inhibitions — the dead-cat filter — already applied); bounded
    /// keeps the strongest verdict's stability.
    fn fold(signals: Option<&Vec<Value>>) -> ChannelReading {
        let mut r = ChannelReading::default();
        let Some(signals) = signals else { return r };
        for s in signals {
            let kind = s.get("kind").and_then(Value::as_str).unwrap_or("");
            let raw = s.get("intensity").and_then(Value::as_f64).unwrap_or(0.0);
            let eff = s
                .get("effective_intensity")
                .and_then(Value::as_f64)
                .unwrap_or(raw);
            match kind {
                "gold" => r.gold += eff,
                "bounded" => r.bounded = r.bounded.max(raw / 10.0),
                "taboo" => r.taboo = r.taboo.max(raw),
                "dead-end" => r.dead_end += raw,
                "help" => r.help += raw,
                _ => {}
            }
        }
        r
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signatures_compose_the_figure() {
        // bright present: gold on firm ground, no shadow
        let bright = ChannelReading {
            gold: 6.0,
            bounded: 0.8,
            taboo: 0.0,
            dead_end: 0.0,
            help: 0.0,
        };
        assert_eq!(bright.signature(), 0b1100);

        // shadowed past: taboo over failures
        let shadow = ChannelReading {
            gold: 0.0,
            bounded: 0.1,
            taboo: 7.0,
            dead_end: 4.0,
            help: 1.0,
        };
        assert_eq!(shadow.signature(), 0b0011);

        // the figure: present over past → one of the 256, deterministically
        let binary = (bright.signature() << 4) | shadow.signature();
        assert_eq!(binary, 0b1100_0011);
        let odu = get_odu_by_binary(binary);
        assert_eq!(odu.binary, binary);
    }

    #[test]
    fn dead_end_bit_requires_losses_to_outnumber_help() {
        let contested = ChannelReading {
            gold: 0.0,
            bounded: 0.0,
            taboo: 0.0,
            dead_end: 2.0,
            help: 3.0,
        };
        assert_eq!(contested.signature(), 0b0000, "help outweighs loss");
    }

    #[test]
    fn bounded_quartiles_span_the_tetragrams() {
        let quart = |s: f64| -> u8 {
            match (s * 4.0) as u8 {
                0 => 0b0000,
                1 => 0b0001,
                2 => 0b0011,
                3 => 0b0111,
                _ => 0b1111,
            }
        };
        assert_eq!(quart(0.05), 0b0000);
        assert_eq!(quart(0.3), 0b0001);
        assert_eq!(quart(0.6), 0b0011);
        assert_eq!(quart(0.8), 0b0111);
        assert_eq!(quart(1.0), 0b1111);
    }
}
