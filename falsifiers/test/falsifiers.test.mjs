/**
 * Falsifier tests — run each module in a mock of crucible-probe's sandbox.
 *
 *   node --test test/falsifiers.test.mjs
 *
 * Two properties get as much attention as the happy paths, because both are
 * ways a falsifier can be wrong while looking right:
 *
 *  1. **Abstention.** A module that cannot see must emit `indeterminate`, never
 *     `fails`. The kernel weighs a verdict as evidence either way, so a probe
 *     that treats blindness as refutation manufactures disagreement.
 *  2. **Non-vacuity.** `claim.build` refuses a pure falsifier whose outcome
 *     never moves under mutation of its inputs — a module that returns `holds`
 *     unconditionally is worse than no falsifier, because it looks like one.
 */

import test from 'node:test';
import assert from 'node:assert/strict';
import { runFalsifier, canonicalMutations, HOLDS, FAILS, INDETERMINATE } from './harness.mjs';

const DIR = 'target/wasm32-unknown-unknown/release/';
const GATES = DIR + 'gates_passed.wasm';
const DETERM = DIR + 'deterministic_execution.wasm';
const ENFORCE = DIR + 'enforcement_proportionate.wasm';
const SIGNAL = DIR + 'signal_resolved.wasm';
const MARKET = DIR + 'market_resolved.wasm';

const run = (m, inputs, extra) => runFalsifier(m, Object.assign({ inputs }, extra || {}));

// === gates-passed ===

test('gates-passed: holds when the record says the gates passed', () => {
  assert.equal(run(GATES, { gates_passed: true, odu_index: 7 }).verdict, HOLDS);
});

test('gates-passed: refutes a claim of sanction the record contradicts', () => {
  // The failure worth catching: an agent asserting it was sanctioned when the
  // accompanying record says otherwise.
  assert.equal(run(GATES, { gates_passed: false, odu_index: 7 }).verdict, FAILS);
});

test('gates-passed: refutes an odu index outside the corpus', () => {
  // The boolean saying true does not rescue an internally incoherent record.
  assert.equal(run(GATES, { gates_passed: true, odu_index: 300 }).verdict, FAILS);
  assert.equal(run(GATES, { gates_passed: true, odu_index: -1 }).verdict, FAILS);
});

test('gates-passed: abstains when there is no boolean to read', () => {
  assert.equal(run(GATES, { vessel: 'Ogun' }).verdict, INDETERMINATE);
});

// === deterministic-execution ===

test('deterministic-execution: holds when a replay reproduces the root', () => {
  const r = run(DETERM, { state_root: 'ab12', replay_root: 'ab12', deterministic: true });
  assert.equal(r.verdict, HOLDS);
});

test('deterministic-execution: refutes when the replay diverges', () => {
  assert.equal(run(DETERM, { state_root: 'ab12', replay_root: 'ffff' }).verdict, FAILS);
});

test('deterministic-execution: abstains when no replay was supplied', () => {
  // Load-bearing. Reporting `holds` here would let an agent earn belief by
  // omitting the evidence rather than producing it.
  assert.equal(run(DETERM, { state_root: 'ab12', deterministic: true }).verdict, INDETERMINATE);
});

test('deterministic-execution: refutes a record that contradicts itself', () => {
  const r = run(DETERM, { state_root: 'ab12', replay_root: 'ab12', deterministic: false });
  assert.equal(r.verdict, FAILS);
});

test('deterministic-execution: refutes an empty state root', () => {
  assert.equal(run(DETERM, { state_root: '', replay_root: '' }).verdict, FAILS);
});

// === enforcement-proportionate ===

test('enforcement-proportionate: holds when action is within the ceiling', () => {
  assert.equal(run(ENFORCE, { severity: 'severe', action: 'quarantine_state' }).verdict, HOLDS);
  assert.equal(run(ENFORCE, { severity: 'info', action: 'observe' }).verdict, HOLDS);
});

test('enforcement-proportionate: refutes escalation beyond the severity', () => {
  // The assertion a guardian cannot be trusted to self-certify.
  assert.equal(run(ENFORCE, { severity: 'warning', action: 'quarantine_state' }).verdict, FAILS);
  assert.equal(run(ENFORCE, { severity: 'info', action: 'rollback_transition' }).verdict, FAILS);
});

test('enforcement-proportionate: does NOT refute restraint', () => {
  // Under-reacting is not a false proportionality claim, and punishing it
  // would push the ladder in exactly the wrong direction.
  assert.equal(run(ENFORCE, { severity: 'critical', action: 'observe' }).verdict, HOLDS);
});

test('enforcement-proportionate: refutes uncorroborated rollback', () => {
  // Irreversible action demands a co-signature regardless of severity.
  const bare = { severity: 'critical', action: 'rollback_transition', signature_count: 0 };
  assert.equal(run(ENFORCE, bare).verdict, FAILS);
  const signed = { severity: 'critical', action: 'rollback_transition', signature_count: 2 };
  assert.equal(run(ENFORCE, signed).verdict, HOLDS);
});

test('enforcement-proportionate: abstains on a ladder it was not taught', () => {
  // An unrecognised rung is a vocabulary gap, not a violation.
  const r = run(ENFORCE, { severity: 'apocalyptic', action: 'observe' });
  assert.equal(r.verdict, INDETERMINATE);
});

// === signal-resolved (observational) ===

const MANIFEST = { observations: ['market:close'] };

test('signal-resolved: holds when the close moved as called', () => {
  const r = run(SIGNAL, { direction: 'long', entry: 142.5 },
    { manifest: MANIFEST, observations: { 'market:close': '150.0' } });
  assert.equal(r.verdict, HOLDS);
});

test('signal-resolved: refutes when the close moved against the call', () => {
  const r = run(SIGNAL, { direction: 'long', entry: 142.5 },
    { manifest: MANIFEST, observations: { 'market:close': '130.0' } });
  assert.equal(r.verdict, FAILS);
});

test('signal-resolved: handles short in the opposite sense', () => {
  const opts = { manifest: MANIFEST, observations: { 'market:close': '130.0' } };
  assert.equal(run(SIGNAL, { direction: 'short', entry: 142.5 }, opts).verdict, HOLDS);
});

test('signal-resolved: abstains when the observation was never gathered', () => {
  // A relay outage must not manufacture a refutation.
  const r = run(SIGNAL, { direction: 'long', entry: 142.5 }, { manifest: MANIFEST });
  assert.equal(r.verdict, INDETERMINATE);
});

test('signal-resolved: declares exactly one observation', () => {
  const r = run(SIGNAL, { direction: 'long', entry: 142.5 },
    { manifest: MANIFEST, observations: { 'market:close': '150.0' } });
  assert.deepEqual(r.observed, ['market:close'], 'blast radius must be one line');
});

test('signal-resolved: traps if run without its grant', () => {
  // Denial is a trap in the real sandbox, so a module cannot mistake "denied"
  // for "absent" and report a confident verdict about a world it never saw.
  const r = run(SIGNAL, { direction: 'long', entry: 142.5 }, { manifest: { observations: [] } });
  assert.equal(r.verdict, 'trapped');
  assert.match(r.message, /not granted by the manifest/);
});

// === market-resolved (observational) ===
//
// The venue settles the claim, so the claimant cannot tune the outcome. What
// these cover is the part that is easy to get wrong: the several ways a market
// can decline to give an answer, none of which are refutations.

const MKT = { observations: ['market:resolution'] };
const mkt = (resolution) =>
  resolution === undefined ? { manifest: MKT } : { manifest: MKT, observations: { 'market:resolution': resolution } };

test('market-resolved: holds when the market settled as claimed', () => {
  assert.equal(run(MARKET, { outcome: 'yes' }, mkt('yes')).verdict, HOLDS);
});

test('market-resolved: refutes when the market settled the other way', () => {
  assert.equal(run(MARKET, { outcome: 'yes' }, mkt('no')).verdict, FAILS);
});

test('market-resolved: outcome names are compared case-insensitively', () => {
  assert.equal(run(MARKET, { outcome: 'Yes' }, mkt('YES')).verdict, HOLDS);
});

test('market-resolved: distinct outcomes sharing a prefix do not agree', () => {
  // Truncating the comparison would report these as equal.
  assert.equal(run(MARKET, { outcome: 'yes' }, mkt('yes-but-void')).verdict, FAILS);
});

for (const state of ['unresolved', 'pending', 'open', 'void', 'cancelled', 'canceled']) {
  test(`market-resolved: abstains when the market is ${state}`, () => {
    // A withdrawn or unsettled question is not a wrong answer. Reporting these
    // as FAILS would let a venue cancel a market into a refutation.
    assert.equal(run(MARKET, { outcome: 'yes' }, mkt(state)).verdict, INDETERMINATE);
  });
}

test('market-resolved: abstains when the resolution was never gathered', () => {
  assert.equal(run(MARKET, { outcome: 'yes' }, mkt()).verdict, INDETERMINATE);
});

test('market-resolved: abstains when the claim names no outcome', () => {
  assert.equal(run(MARKET, { venue: 'limitless' }, mkt('yes')).verdict, INDETERMINATE);
});

test('market-resolved: abstains rather than truncating an oversized outcome', () => {
  // No allocator: refusing is the only safe answer, since a truncated compare
  // invents agreement between different outcomes.
  const long = 'y'.repeat(200);
  assert.equal(run(MARKET, { outcome: long }, mkt('yes')).verdict, INDETERMINATE);
});

test('market-resolved: declares exactly one observation', () => {
  const r = run(MARKET, { outcome: 'yes' }, mkt('yes'));
  assert.deepEqual(r.observed, ['market:resolution'], 'blast radius must be one line');
});

test('market-resolved: traps if run without its grant', () => {
  const r = run(MARKET, { outcome: 'yes' }, { manifest: { observations: [] } });
  assert.equal(r.verdict, 'trapped');
  assert.match(r.message, /not granted by the manifest/);
});

test('market-resolved: the verdict tracks the settlement, not the inputs', () => {
  // The property that makes this a real falsifier rather than one hiding behind
  // the impure-module exemption: hold the claim fixed, move the world, and the
  // verdict moves with it.
  const claim = { outcome: 'yes' };
  assert.equal(run(MARKET, claim, mkt('yes')).verdict, HOLDS);
  assert.equal(run(MARKET, claim, mkt('no')).verdict, FAILS);
});

// === non-vacuity: what claim.build enforces on pure falsifiers ===

for (const [name, mod, inputs] of [
  ['gates-passed', GATES, { gates_passed: true, odu_index: 7 }],
  ['deterministic-execution', DETERM, { state_root: 'ab12', replay_root: 'ab12' }],
  ['enforcement-proportionate', ENFORCE, { severity: 'severe', action: 'quarantine_state' }],
]) {
  test(`${name}: outcome moves under canonical mutation (non-vacuous)`, () => {
    const base = run(mod, inputs).verdict;
    const moved = canonicalMutations(inputs).some((m) => run(mod, m).verdict !== base);
    assert.ok(moved, `${name} returns ${base} regardless of its inputs — claim.build would refuse it`);
  });
}

test('every module exports the ABI the sandbox requires', () => {
  // A module missing either export is rejected before it runs; catching that
  // here beats catching it at a relay.
  for (const m of [GATES, DETERM, ENFORCE, SIGNAL, MARKET]) {
    assert.doesNotThrow(() => run(m, {}), `${m} failed to instantiate`);
  }
});
