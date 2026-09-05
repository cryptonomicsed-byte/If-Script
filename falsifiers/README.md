# Crucible falsifiers for the Technosis ecosystem

Every Crucible claim carries a falsifier: a content-addressed WASM predicate
that returns false if the claim is false. Without one a claim is rejected at
parse time — that is Crucible's single rule, and it is why the claim paths
across this ecosystem all required a falsifier and, until now, had none to
supply. Claims could be built and not usefully resolved.

These are those modules.

## What is here

| Module | Claim it falsifies | Pure? | Emitted by |
|---|---|---|---|
| `gates_passed` | the ritual's governance gates passed | yes | IfáScript, Kóòdù |
| `deterministic_execution` | the job executed deterministically | yes | Ọ̀ṢỌ́VM |
| `enforcement_proportionate` | the enforcement matched the anomaly | yes | Zàngbétò |
| `signal_resolved` | the directional call was correct | **no** | Loom |
| `market_resolved` | the market settled as the claim said | **no** | Vantage, Loom |

## Content addresses

Rebuild with `./build.sh`. A claim references the address, so it must be
reproducible — if it moves, every claim pointing at the old one dangles.

```
gates_passed              sha256:18d904870ba845d962ac72cb9cc3198b877c28e0257fac839bc51c1d4e558994
deterministic_execution   sha256:975d498fa6991d920b02e26be00fa771f0d1cb4109ae44c2d3a7d12b77c0a9ed
enforcement_proportionate sha256:2f628270fbc78111b3f6e2172f4f78840ec39f4c8d84f8db84a4382992e41547
signal_resolved           sha256:bc150381144666386b67fa9d95ed96e8f3e5590ab926112c4a1b7a3e66394a41
market_resolved           sha256:d4c928dc5e988b9f4d9a46272c6093d808a22963093b8e9f736feae77aa04af1
```

## Manifests

The manifest is the blast radius, readable before anyone runs the module.

```json
gates_passed              {"observations": []}
deterministic_execution   {"observations": []}
enforcement_proportionate {"observations": []}
signal_resolved           {"observations": ["market:close"]}
market_resolved           {"observations": ["market:resolution"]}
```

Three are **pure**: closed computations over the claim's own declared inputs.
Two agents anywhere must get byte-identical output, and a disagreement is a
defect rather than a difference of opinion about the world.

Two are **observational** — whether a prediction came true is a fact about the
world, not about the record. Each manifest grants exactly one observation, so a
reviewer reads the whole blast radius in a line. That is the entire argument
for declaring capabilities instead of trusting a module's self-description.

`market_resolved` is the stronger of the two, because a market settles at a
stated time under a party the claimant does not control. `audit_vacuity`
*exempts* impure falsifiers rather than testing them, so that exemption is a
concession a module could hide behind; this one does not, and its tests hold
the claim fixed while moving the world to prove the verdict follows.

## Two properties that get as much care as the logic

**Abstention.** A module that cannot see emits `indeterminate`, never `fails`.
The kernel weighs a verdict as evidence either way, so a probe that treats
blindness as refutation lets a relay outage manufacture disagreement. Every
"I couldn't read that" path in these modules ends in `indeterminate`, and the
tests assert it.

**Non-vacuity.** `claim.build` runs `audit_vacuity` against a pure falsifier:
it re-runs the module under structural mutations of its inputs and refuses the
claim if the outcome never moves. A module returning `holds` unconditionally is
worse than no falsifier, because it looks like one. `test/falsifiers.test.mjs`
runs the same mutations (ported from `crucible-probe/src/vacuity.rs`) so a
regression is caught here rather than at claim construction.

## What they deliberately do not do

`enforcement_proportionate` does **not** refute restraint. A guardian that
observed where it could have quarantined has made no false claim about
proportionality, and refuting it would push the ladder toward escalation.

`deterministic_execution` does **not** report `holds` when no replay was
supplied. Nothing has been checked in that case, and crediting it would let an
agent earn belief by withholding evidence rather than producing it.

`gates_passed` does **not** re-derive Kóòdù's window arithmetic. It cannot see
the calendar and must not guess at it; it checks the narrower thing that is
actually checkable — that the claim does not contradict its own record.

## Testing

```sh
./build.sh
node --test test/falsifiers.test.mjs
```

`test/harness.mjs` is a mock of `crucible-probe`'s sandbox, mirrored from
`sandbox.rs` rather than from prose. It reproduces the two host behaviours
easiest to get wrong: an ungranted observation **traps** rather than returning
a sentinel, and a granted-but-ungathered one returns `-1`. A guest that
mistook denial for absence would report a confident verdict about a world it
never saw.

## Writing another one

1. Add `src/bin/<name>.rs`, `#![no_std] #![no_main]`, export `crucible_falsify`.
2. Every path must reach an `emit` — returning without one is *inconclusive*,
   not "holds".
3. Grant the minimum observations, or none.
4. Add cases to the test file, including a `cannot see → indeterminate` case
   and, if pure, the non-vacuity check.
5. Rebuild, publish the new address, update the table above.
