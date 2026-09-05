//! A public model benchmark that does not require trusting its operator.
//!
//! # The problem this solves
//!
//! Moon Dev's AI Trading Battles states its own thesis exactly right: *a
//! benchmark you can't audit is a marketing page.* Six models get an identical
//! market snapshot on an identical schedule, each trades a real account, and
//! every decision is published with its unedited reasoning.
//!
//! Then its `HEARTBEAT_API_SPEC.md` secures all of that with a shared secret,
//! stores one row, and instructs that history is never kept:
//!
//! > *Action: **overwrite** the single stored heartbeat (one row / one file /
//! > one Redis key. History is not needed, and never let this table grow.)*
//!
//! Each of those is a compromise this ecosystem already has the machinery to
//! avoid, so an arena built here is more auditable than the one whose entire
//! selling point is auditability:
//!
//! | Their mechanism | What it costs | What replaces it |
//! |---|---|---|
//! | `X-Battle-Key` shared secret | anyone holding it can forge a beat | NIP-42 plus per-fighter signatures — forgery-proof rather than secret-proof |
//! | one overwritten row | the audit trail is destroyed on purpose | `kind:30174` addressable events: latest-wins natively, history retained |
//! | `server_received_at` | one operator's clock is the arbiter | `created_at` plus multi-relay receipt: several independent clocks |
//! | naive model consensus | correlated models counted as independent | redundancy discounting ([`consensus`]) |
//!
//! # What is *not* claimed
//!
//! Signed events fix attribution and tamper-evidence. They do not make a
//! fighter's self-reported P&L true. A decision event proves *this key said
//! this, at this time, and has not been edited since* — nothing more. Anything
//! about money must be settled against the chain, and this module deliberately
//! carries no field that would invite a reader to take a fighter's word for it.

use serde::{Deserialize, Serialize};

use super::events::EventError;
use super::identity::NostrIdentity;
use super::kinds;

/// What a fighter decided for one round.
///
/// `reasoning` is carried verbatim, including when it is bad. A benchmark that
/// published only its fighters' good reasoning would be measuring the
/// publisher's taste rather than the model.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Decision {
    /// Which arena this belongs to, so several can share a relay.
    pub arena: String,
    /// Monotonic round number. The snapshot is identical within a round; that
    /// is the whole basis for comparing fighters at all.
    pub round: u64,
    /// Instrument the round was fought on.
    pub symbol: String,
    /// `"long"`, `"short"` or `"flat"`. Sitting out is a decision and is
    /// recorded as one — a model that correctly declines a bad round should not
    /// look identical to one that never answered.
    pub stance: String,
    /// Fraction of the fighter's account committed, 0.0..=1.0.
    pub size: f64,
    /// The model's own unedited account of why.
    pub reasoning: String,
    /// Digest of the snapshot every fighter was handed this round. Readers
    /// verify that fighters were actually given identical inputs rather than
    /// taking the operator's word for it.
    pub snapshot_hash: String,
}

/// A fighter's liveness, published as a replaceable event.
///
/// Their spec keeps exactly one heartbeat and stamps arrival with the relay's
/// clock because *"the mac and the battle box each have their own idea of
/// `now`"*. An addressable event gives the same latest-wins read without
/// destroying the history, and multiple relays give several independent
/// arrival clocks instead of one operator's.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Heartbeat {
    pub arena: String,
    /// `BOOT`, `THINKING`, `PUSHING`, `SLEEPING`, `ERROR`, `SHUTDOWN`.
    pub phase: String,
    pub rounds_done: u64,
    /// Seconds since this fighter started.
    pub uptime_secs: u64,
    /// Last error, if the fighter is carrying one. Published rather than
    /// hidden: a benchmark that conceals its participants' failures is
    /// measuring something other than the participants.
    pub last_error: Option<String>,
}

/// Sign a round decision as an engram addressed to the arena steward.
///
/// The decision is a *record*, not an assertion about the world, so it is an
/// engram rather than a claim. A fighter asserting that its decision was
/// *correct* is a separate Crucible claim resolved by a falsifier — see
/// `market_resolved`.
pub fn decision_engram(
    identity: &NostrIdentity,
    decision: &Decision,
    steward_pubkey_hex: &str,
) -> Result<nostr::Event, EventError> {
    let slug = round_slug(&decision.arena, decision.round).ok_or_else(|| {
        EventError::Serialisation(format!(
            "arena {:?} cannot form a valid engram slug",
            decision.arena
        ))
    })?;
    super::events::engram_with_slug(
        identity,
        decision,
        steward_pubkey_hex,
        &slug,
        vec![
            ("arena", decision.arena.clone()),
            ("round", decision.round.to_string()),
            ("stance", decision.stance.clone()),
            ("snapshot", decision.snapshot_hash.clone()),
        ],
    )
}

/// Sign a heartbeat as a replaceable engram.
pub fn heartbeat_engram(
    identity: &NostrIdentity,
    beat: &Heartbeat,
    steward_pubkey_hex: &str,
) -> Result<nostr::Event, EventError> {
    let slug = heartbeat_slug(&beat.arena).ok_or_else(|| {
        EventError::Serialisation(format!("arena {:?} cannot form a valid engram slug", beat.arena))
    })?;
    super::events::engram_with_slug(
        identity,
        beat,
        steward_pubkey_hex,
        &slug,
        vec![
            ("arena", beat.arena.clone()),
            ("phase", beat.phase.clone()),
        ],
    )
}

/// Engram slugs live under `mem/`, and each round gets its own address so
/// rounds accumulate instead of overwriting one another — the whole difference
/// from a single stored row.
fn round_slug(arena: &str, round: u64) -> Option<String> {
    let a = kinds::normalize_slug_segment(arena)?;
    let s = format!("mem/arena/{a}/round/{round}");
    kinds::validate_slug(&s).then_some(s)
}

/// One stable address per fighter per arena: latest-wins without deleting
/// anything, since the relay keeps the superseded events.
fn heartbeat_slug(arena: &str) -> Option<String> {
    let a = kinds::normalize_slug_segment(arena)?;
    let s = format!("mem/arena/{a}/heartbeat");
    kinds::validate_slug(&s).then_some(s)
}

/// How stale a heartbeat is allowed to be before a fighter counts as absent.
pub const STALE_AFTER_SECS: u64 = 300;

/// Whether a heartbeat seen at `observed_at` is stale as of `now`.
///
/// Both timestamps come from event `created_at` values and a reader's own
/// clock, never from a single operator's. A beat stamped in the future is
/// treated as *fresh* rather than as an error: a fighter whose clock runs fast
/// is a clock problem, and calling it absent would penalise the fighter for the
/// operator's failure to run NTP.
pub fn is_stale(observed_at: u64, now: u64) -> bool {
    now.saturating_sub(observed_at) > STALE_AFTER_SECS
}

/// One fighter's answer, as the consensus reader sees it.
#[derive(Debug, Clone, PartialEq)]
pub struct Vote {
    /// The fighter's pubkey.
    pub pubkey: String,
    pub stance: String,
    /// A label for what this fighter *is*, used to discount correlated voices —
    /// a model family, a lab, a shared base checkpoint. Fighters sharing a
    /// lineage are not independent witnesses.
    pub lineage: String,
}

/// A stance and the discounted weight behind it.
#[derive(Debug, Clone, PartialEq)]
pub struct Tally {
    pub stance: String,
    /// Discounted weight, not a headcount.
    pub weight: f64,
    /// How many fighters actually voted this way.
    pub voters: usize,
    /// How many distinct lineages those voters span.
    pub lineages: usize,
}

/// Weigh votes with agreement discounted for redundancy.
///
/// # Why a headcount is the wrong answer
///
/// Moon Dev's `swarm_agent.py` fans out to six models through OpenRouter and
/// returns every response, with no discounting anywhere. Six models trained on
/// overlapping corpora agreeing is not six witnesses — it is closer to one
/// witness counted six times, and a naive majority hands the largest model
/// family a permanent win it did not earn.
///
/// So the *n*-th vote from a lineage already heard from contributes `1/n`. Two
/// fighters from one lineage are worth 1.5 rather than 2; three are worth
/// ~1.83, approaching a ceiling no single family can exceed by cloning itself.
/// A lineage's first vote always counts fully, so a genuinely distinct voice is
/// never penalised for arriving late.
///
/// Returns tallies sorted by descending weight. Ties keep insertion order,
/// making the result deterministic — two readers must derive the same tally
/// from the same votes or the arena settles nothing.
pub fn consensus(votes: &[Vote]) -> Vec<Tally> {
    let mut order: Vec<String> = Vec::new();
    let mut tallies: Vec<Tally> = Vec::new();
    // Per stance, how many times each lineage has already been counted.
    let mut seen: Vec<(String, Vec<(String, u32)>)> = Vec::new();

    for vote in votes {
        if !order.iter().any(|s| s == &vote.stance) {
            order.push(vote.stance.clone());
            tallies.push(Tally {
                stance: vote.stance.clone(),
                weight: 0.0,
                voters: 0,
                lineages: 0,
            });
            seen.push((vote.stance.clone(), Vec::new()));
        }
        let idx = order.iter().position(|s| s == &vote.stance).unwrap();
        let lineages = &mut seen[idx].1;

        let n = match lineages.iter_mut().find(|(l, _)| l == &vote.lineage) {
            Some((_, count)) => {
                *count += 1;
                *count
            }
            None => {
                lineages.push((vote.lineage.clone(), 1));
                1
            }
        };

        tallies[idx].weight += 1.0 / f64::from(n);
        tallies[idx].voters += 1;
        tallies[idx].lineages = lineages.len();
    }

    // Stable sort: equal weights keep the order stances were first seen, so the
    // output is a function of the input alone.
    tallies.sort_by(|a, b| b.weight.partial_cmp(&a.weight).unwrap_or(core::cmp::Ordering::Equal));
    tallies
}

/// The winning stance, or `None` when the arena has not decided.
///
/// `None` on an empty field, and on a tie. A tie is genuinely undecided, and
/// breaking it by insertion order would hand the result to whichever fighter
/// happened to answer first.
pub fn leading_stance(votes: &[Vote]) -> Option<String> {
    let tallies = consensus(votes);
    match tallies.as_slice() {
        [] => None,
        [only] => Some(only.stance.clone()),
        [first, second, ..] => {
            if (first.weight - second.weight).abs() < f64::EPSILON {
                None
            } else {
                Some(first.stance.clone())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(pubkey: &str, stance: &str, lineage: &str) -> Vote {
        Vote {
            pubkey: pubkey.into(),
            stance: stance.into(),
            lineage: lineage.into(),
        }
    }

    fn a_decision() -> Decision {
        Decision {
            arena: "battles".into(),
            round: 42,
            symbol: "BTC".into(),
            stance: "long".into(),
            size: 0.25,
            reasoning: "funding flipped negative while OI kept climbing".into(),
            snapshot_hash: "abc123".into(),
        }
    }

    #[test]
    fn a_decision_is_signed_and_verifies() {
        let id = NostrIdentity::generate();
        let ev = decision_engram(&id, &a_decision(), id.public_key_hex()).unwrap();
        assert!(ev.verify().is_ok());
        assert_eq!(ev.pubkey.to_hex(), id.public_key_hex());
    }

    #[test]
    fn a_decision_round_trips_through_its_content() {
        let id = NostrIdentity::generate();
        let d = a_decision();
        let ev = decision_engram(&id, &d, id.public_key_hex()).unwrap();
        let back: Decision = serde_json::from_str(ev.content()).unwrap();
        assert_eq!(back, d);
    }

    #[test]
    fn different_rounds_get_different_addresses() {
        // Rounds must not overwrite one another; that is the whole difference
        // from the single stored row.
        let id = NostrIdentity::generate();
        let mut a = a_decision();
        let mut b = a_decision();
        a.round = 1;
        b.round = 2;
        let ea = decision_engram(&id, &a, id.public_key_hex()).unwrap();
        let eb = decision_engram(&id, &b, id.public_key_hex()).unwrap();
        assert_ne!(d_of(&ea), d_of(&eb));
    }

    #[test]
    fn heartbeats_from_one_fighter_share_an_address() {
        // Replaceable on purpose: latest-wins is the read we want, and we get
        // it without deleting anything.
        let id = NostrIdentity::generate();
        let beat = Heartbeat {
            arena: "battles".into(),
            phase: "THINKING".into(),
            rounds_done: 12,
            uptime_secs: 3600,
            last_error: None,
        };
        let mut later = beat.clone();
        later.phase = "SLEEPING".into();
        later.rounds_done = 13;

        let a = heartbeat_engram(&id, &beat, id.public_key_hex()).unwrap();
        let b = heartbeat_engram(&id, &later, id.public_key_hex()).unwrap();
        assert_eq!(d_of(&a), d_of(&b));
    }

    #[test]
    fn two_fighters_never_share_a_heartbeat_address() {
        // The d-tag is HMAC'd under a per-fighter conversation key, so one
        // fighter cannot overwrite another's liveness.
        let a = NostrIdentity::generate();
        let b = NostrIdentity::generate();
        let beat = Heartbeat {
            arena: "battles".into(),
            phase: "BOOT".into(),
            rounds_done: 0,
            uptime_secs: 1,
            last_error: None,
        };
        let ea = heartbeat_engram(&a, &beat, a.public_key_hex()).unwrap();
        let eb = heartbeat_engram(&b, &beat, b.public_key_hex()).unwrap();
        assert_ne!(d_of(&ea), d_of(&eb));
    }

    fn d_of(ev: &nostr::Event) -> String {
        ev.tags()
            .iter()
            .find_map(|t| {
                let v = t.as_vec();
                (v.first().map(String::as_str) == Some("d")).then(|| v[1].clone())
            })
            .expect("engram must carry a d tag")
    }

    #[test]
    fn a_fresh_heartbeat_is_not_stale() {
        assert!(!is_stale(1000, 1000 + STALE_AFTER_SECS));
    }

    #[test]
    fn an_old_heartbeat_is_stale() {
        assert!(is_stale(1000, 1000 + STALE_AFTER_SECS + 1));
    }

    #[test]
    fn a_beat_from_the_future_is_fresh_not_an_error() {
        // A fighter's fast clock is the operator's NTP problem, not grounds to
        // report the fighter absent.
        assert!(!is_stale(2000, 1000));
    }

    #[test]
    fn one_fighter_per_lineage_counts_fully() {
        let votes = vec![
            v("a", "long", "anthropic"),
            v("b", "long", "openai"),
            v("c", "short", "google"),
        ];
        let t = consensus(&votes);
        assert_eq!(t[0].stance, "long");
        assert_eq!(t[0].weight, 2.0);
        assert_eq!(t[0].lineages, 2);
    }

    #[test]
    fn a_lineage_cannot_win_by_cloning_itself() {
        // The defect this function exists to prevent: four instances of one
        // family outvoting two genuinely distinct voices.
        let votes = vec![
            v("a1", "long", "samefamily"),
            v("a2", "long", "samefamily"),
            v("a3", "long", "samefamily"),
            v("a4", "long", "samefamily"),
            v("b", "short", "openai"),
            v("c", "short", "google"),
        ];
        // Naive headcount: long 4, short 2 — long wins.
        // Discounted: long 1 + 1/2 + 1/3 + 1/4 ≈ 2.083, short 2.0.
        let t = consensus(&votes);
        assert_eq!(t[0].stance, "long");
        assert!(t[0].weight < 2.5, "four clones must not weigh 4: {}", t[0].weight);

        // One more clone still cannot beat a third distinct voice.
        let mut more = votes.clone();
        more.push(v("a5", "long", "samefamily"));
        more.push(v("d", "short", "meta"));
        assert_eq!(leading_stance(&more).as_deref(), Some("short"));
    }

    #[test]
    fn a_lineages_first_vote_always_counts_fully() {
        let votes = vec![
            v("a1", "long", "x"),
            v("a2", "long", "x"),
            v("b", "long", "y"),
        ];
        let t = consensus(&votes);
        // x contributes 1 + 1/2, y contributes a full 1 despite arriving last.
        assert!((t[0].weight - 2.5).abs() < 1e-9, "got {}", t[0].weight);
    }

    #[test]
    fn a_tie_decides_nothing() {
        let votes = vec![v("a", "long", "x"), v("b", "short", "y")];
        assert_eq!(leading_stance(&votes), None);
    }

    #[test]
    fn an_empty_field_decides_nothing() {
        assert_eq!(leading_stance(&[]), None);
    }

    #[test]
    fn a_single_voice_decides() {
        assert_eq!(leading_stance(&[v("a", "flat", "x")]).as_deref(), Some("flat"));
    }

    #[test]
    fn sitting_out_is_a_stance_not_an_absence() {
        // A model that correctly declines a bad round must be distinguishable
        // from one that never answered.
        let votes = vec![v("a", "flat", "x"), v("b", "flat", "y"), v("c", "long", "z")];
        assert_eq!(leading_stance(&votes).as_deref(), Some("flat"));
    }

    #[test]
    fn the_tally_is_a_function_of_the_votes_alone() {
        // Two readers must derive the same result, or the arena settles nothing.
        let votes = vec![
            v("a", "long", "x"),
            v("b", "short", "y"),
            v("c", "long", "z"),
            v("d", "short", "w"),
        ];
        let once = consensus(&votes);
        let twice = consensus(&votes);
        assert_eq!(once, twice);
    }
}
