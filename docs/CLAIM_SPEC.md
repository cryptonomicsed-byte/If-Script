# Writing a claim an agent can be held to

A Crucible claim (`kind:47001`) is only as good as its statement. A vague
statement cannot be refuted, so it cannot be attested either — it just decays
by half-life while looking like knowledge. This is the shape that avoids that.

## Where the format comes from

Moon Dev's Polymarket bots are unremarkable as code and unusually good as
specification. `5_minute_bots/near_liq_trigger/near_liq_trigger.py` opens with:

```
  1. ARM  — watch for a BTC position within 0.5% of its liquidation price AND
            worth over $100,000. Whichever SIDE has the closest such position
            tells us where price is headed.
  2. TRIGGER — do NOT trade on the arm alone. Wait until somebody on that SAME
            side actually gets liquidated for at least $5,000.
  3. FIRE  — take the ask on the 5-minute BTC up/down market, $5 flat.
  4. HOLD  — there is no exit. We hold to expiration.

BTC ONLY. Most 5-minute windows this bot does NOTHING — it just sits there
armed (or not armed) and waits. That is the design, not a bug.
```

Every threshold is a named constant with a stated rationale, and the last
paragraph is the part most specifications omit: **what it looks like when
nothing happens**. Without that line, a reader cannot distinguish a strategy
correctly declining from a strategy that is broken.

## The five parts

A claim statement carries all five. Anything missing makes the claim weaker
than it looks.

| Part | What it fixes | Failure if omitted |
|---|---|---|
| **Precondition** | the world-state that makes the claim apply at all | the claim is evaluated in conditions it never addressed |
| **Trigger** | the observable event that starts the clock | "eventually" — unfalsifiable, because it never expires |
| **Assertion** | what is claimed to follow, with a bound | vague direction with no threshold to miss |
| **Horizon** | when it resolves | a claim that cannot be wrong yet cannot be right yet |
| **Null behaviour** | what "nothing happened" looks like | inaction gets scored as failure |

## Worked example

```json
{
  "statement": "PRECONDITION: a BTC perp position within 0.5% of liquidation and worth over $100,000 exists on the long side. TRIGGER: a long liquidation print of at least $5,000 lands within 120s. ASSERTION: the Limitless 5-minute BTC market resolves 'no' (down). HORIZON: that market's stated expiry. NULL: in most windows the precondition never holds and no claim is made; absence of a claim is not a failed claim.",
  "falsifier": "sha256:d4c928dc5e988b9f4d9a46272c6093d808a22963093b8e9f736feae77aa04af1",
  "odu_index": 7,
  "half_life_secs": 3600
}
```

with the probe gathering:

```json
{"observations": {"market:resolution": "no"}}
```

## Rules the parts have to obey

**The horizon must be shorter than the half-life.** Belief decays on the
half-life; if the claim cannot resolve before it decays, it will never be
attested while anyone still cares. `market_resolved` claims should carry a
half-life on the order of the market's own duration.

**The assertion must name something the falsifier can read.** Writing "the
market resolves down" when the falsifier compares against `"no"` produces a
`Fails` on a claim that was right. The statement is prose for humans; the
`outcome` field is what is actually compared, and they must agree.

**The null behaviour is not optional.** An agent that publishes a claim every
round regardless of preconditions is not being rigorous, it is being noisy, and
Crucible's redundancy discounting will treat the flood as one voice anyway.

**Do not claim what a falsifier cannot settle.** "This strategy is profitable"
has no module behind it. "This market resolves `no`" does. If no falsifier
exists for an assertion, that assertion is not yet a claim — it is a note, and
belongs in an engram.

## Choosing the falsifier

| If the assertion is about... | Use | Pure? |
|---|---|---|
| governance gates on a cast | `gates_passed` | yes |
| a job replaying identically | `deterministic_execution` | yes |
| enforcement matching an anomaly | `enforcement_proportionate` | yes |
| a directional call against a close | `signal_resolved` | no (`market:close`) |
| a prediction market's settlement | `market_resolved` | no (`market:resolution`) |

Prefer a resolving market where one exists. A pure falsifier settles from the
claim's own record, which makes it reproducible but also means the claimant
supplied everything it reads. A market settles under a party the claimant does
not control — the difference between checking your own work and being marked.
