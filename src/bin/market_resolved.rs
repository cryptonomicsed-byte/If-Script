//! Limitless prediction-market resolution — the **gatherer**.
//!
//! Observation key produced: "market:resolution"
//!
//! # Which of the two market_resolved artifacts this is
//!
//! There are two, and they are different layers rather than duplicates:
//!
//! * **This binary** fetches a market's settlement over the network and prints
//!   a verdict. It is a gatherer and an operator tool.
//! * **`falsifiers/src/bin/market_resolved.rs`** is the sandboxed module
//!   Crucible actually executes. It is `no_std` wasm, content-addressed, and
//!   makes no network call at all — it reads `market:resolution` through the
//!   `observe()` ABI and traps if its manifest does not grant that key.
//!
//! The split is not incidental. A falsifier's manifest is its blast radius, and
//! a reviewer decides whether to run one by reading it. A module that opens its
//! own socket has an unbounded, undeclared blast radius, so the sandboxed form
//! cannot fetch and this form cannot be the thing an attester runs. Feed this
//! binary's output into the module as an observation.
//!
//! Reads a market's outcome from the Limitless public API (no API key) and
//! emits a Crucible verdict:
//!
//!   Passes       — market resolved AND outcome matches the claim
//!   Fails        — market resolved AND outcome does NOT match the claim
//!   Indeterminate — market not yet resolved, or the API was unreachable
//!
//! Blindness discipline: a relay outage or network error always produces
//! Indeterminate — never Fails. The claimant chose an objective, external
//! resolution event; we do not manufacture verdicts from our own silence.
//!
//! Usage:
//!   market_resolved --slug <slug> --outcome <yes|no> [--api <base-url>]
//!
//! Exit codes:
//!   0  Passes
//!   1  Fails
//!   2  Indeterminate
//!   3  Argument / usage error

use clap::Parser;
use serde::Deserialize;

const LIMITLESS_API: &str = "https://api.limitless.exchange"; // VERIFIED_ON: 2026-09-04

#[derive(Parser, Debug)]
#[command(
    name = "market_resolved",
    about = "Crucible falsifier: check Limitless market resolution"
)]
struct Args {
    /// Market slug (e.g. \"will-btc-reach-100k-by-eoy\")
    #[arg(long)]
    slug: String,

    /// Expected outcome: \"yes\" or \"no\"
    #[arg(long)]
    outcome: String,

    /// Limitless API base URL (override for testing)
    #[arg(long, default_value = LIMITLESS_API)]
    api: String,

    /// Output as JSON (default: human-readable)
    #[arg(long)]
    json: bool,
}

#[derive(Debug)]
enum Verdict {
    Passes,
    Fails,
    Indeterminate(String),
}

impl Verdict {
    fn exit_code(&self) -> i32 {
        match self {
            Verdict::Passes => 0,
            Verdict::Fails => 1,
            Verdict::Indeterminate(_) => 2,
        }
    }

    fn label(&self) -> &'static str {
        match self {
            Verdict::Passes => "Passes",
            Verdict::Fails => "Fails",
            Verdict::Indeterminate(_) => "Indeterminate",
        }
    }
}

#[derive(Deserialize, Debug)]
struct MarketResponse {
    #[serde(rename = "isResolved")]
    is_resolved: Option<bool>,
    #[serde(rename = "winningOutcomeIndex")]
    winning_outcome_index: Option<u32>,
    outcomes: Option<Vec<String>>,
    title: Option<String>,
}

fn fetch_resolution(api: &str, slug: &str) -> Result<MarketResponse, String> {
    let url = format!("{}/markets/{}", api, slug);
    let resp = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .user_agent("ifascript-crucible/1.0")
        .build()
        .map_err(|e| format!("client build: {e}"))?
        .get(&url)
        .send()
        .map_err(|e| format!("request failed: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }

    resp.json::<MarketResponse>()
        .map_err(|e| format!("parse error: {e}"))
}

fn evaluate(market: &MarketResponse, claimed_outcome: &str) -> Verdict {
    let resolved = market.is_resolved.unwrap_or(false);
    if !resolved {
        return Verdict::Indeterminate("market has not yet resolved".into());
    }

    let winning_idx = match market.winning_outcome_index {
        Some(i) => i as usize,
        None => return Verdict::Indeterminate("resolved but winning_outcome_index absent".into()),
    };

    let outcomes = match &market.outcomes {
        Some(o) if !o.is_empty() => o,
        _ => return Verdict::Indeterminate("resolved but outcomes list absent".into()),
    };

    let winning_label = match outcomes.get(winning_idx) {
        Some(l) => l.to_lowercase(),
        None => {
            return Verdict::Indeterminate(format!(
                "winning_outcome_index {winning_idx} out of range (len {})",
                outcomes.len()
            ))
        }
    };

    let claimed = claimed_outcome.trim().to_lowercase();
    // Exact match only. A prefix match here reads "yes" as agreeing with a
    // market that settled "yes and no", which manufactures a Passes in the
    // claimant's favour out of a venue's wording. Both sides are already
    // lowercased and trimmed, so "Yes" still matches "yes".
    if claimed == winning_label {
        Verdict::Passes
    } else {
        Verdict::Fails
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn market(resolved: bool, idx: Option<u32>, outcomes: &[&str]) -> MarketResponse {
        MarketResponse {
            is_resolved: Some(resolved),
            winning_outcome_index: idx,
            outcomes: Some(outcomes.iter().map(|s| s.to_string()).collect()),
            title: None,
        }
    }

    #[test]
    fn a_matching_outcome_passes() {
        let m = market(true, Some(0), &["Yes", "No"]);
        assert!(matches!(evaluate(&m, "yes"), Verdict::Passes));
    }

    #[test]
    fn a_differing_outcome_fails() {
        let m = market(true, Some(1), &["Yes", "No"]);
        assert!(matches!(evaluate(&m, "yes"), Verdict::Fails));
    }

    #[test]
    fn outcome_comparison_ignores_case() {
        let m = market(true, Some(0), &["YES", "NO"]);
        assert!(matches!(evaluate(&m, "Yes"), Verdict::Passes));
    }

    #[test]
    fn a_longer_label_sharing_a_prefix_does_not_pass() {
        // The regression this guards: a prefix match turns a venue's wording
        // into a verdict for the claimant.
        let m = market(true, Some(0), &["yes and no", "neither"]);
        assert!(matches!(evaluate(&m, "yes"), Verdict::Fails));
    }

    #[test]
    fn an_unresolved_market_is_indeterminate() {
        let m = market(false, None, &["Yes", "No"]);
        assert!(matches!(evaluate(&m, "yes"), Verdict::Indeterminate(_)));
    }

    #[test]
    fn resolved_without_a_winning_index_is_indeterminate() {
        let m = market(true, None, &["Yes", "No"]);
        assert!(matches!(evaluate(&m, "yes"), Verdict::Indeterminate(_)));
    }

    #[test]
    fn an_out_of_range_index_is_indeterminate() {
        // Never a refutation: an index we cannot resolve is our confusion, not
        // the claimant's error.
        let m = market(true, Some(7), &["Yes", "No"]);
        assert!(matches!(evaluate(&m, "yes"), Verdict::Indeterminate(_)));
    }

    #[test]
    fn an_empty_outcomes_list_is_indeterminate() {
        let m = market(true, Some(0), &[]);
        assert!(matches!(evaluate(&m, "yes"), Verdict::Indeterminate(_)));
    }

    #[test]
    fn every_blind_path_abstains_rather_than_refuting() {
        // The discipline the module header states: silence is never refutation.
        for m in [
            market(false, None, &["Yes", "No"]),
            market(true, None, &["Yes", "No"]),
            market(true, Some(99), &["Yes", "No"]),
            market(true, Some(0), &[]),
        ] {
            assert!(
                !matches!(evaluate(&m, "yes"), Verdict::Fails),
                "a blind path must never produce Fails"
            );
        }
    }
}

fn main() {
    let args = Args::parse();

    let claimed = args.outcome.trim().to_lowercase();
    if claimed != "yes" && claimed != "no" {
        eprintln!("error: --outcome must be 'yes' or 'no', got {:?}", args.outcome);
        std::process::exit(3);
    }

    let verdict = match fetch_resolution(&args.api, &args.slug) {
        Ok(market) => evaluate(&market, &claimed),
        Err(e) => {
            // Blindness discipline: network failure → Indeterminate
            Verdict::Indeterminate(format!("fetch error: {e}"))
        }
    };

    if args.json {
        let reason = match &verdict {
            Verdict::Indeterminate(r) => r.as_str(),
            _ => "",
        };
        println!(
            "{{\"verdict\":\"{}\",\"slug\":\"{}\",\"claimed_outcome\":\"{}\",\"reason\":\"{}\"}}",
            verdict.label(),
            args.slug,
            claimed,
            reason
        );
    } else {
        match &verdict {
            Verdict::Passes => println!("Passes — market resolved, outcome matches '{}'", claimed),
            Verdict::Fails => println!("Fails — market resolved, outcome does NOT match '{}'", claimed),
            Verdict::Indeterminate(r) => println!("Indeterminate — {}", r),
        }
    }

    std::process::exit(verdict.exit_code());
}
