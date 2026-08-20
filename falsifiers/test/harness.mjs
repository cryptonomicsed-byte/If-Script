/**
 * A mock of crucible-probe's sandbox, faithful to the behaviours that matter.
 *
 * Mirrored from `crucible-probe/src/sandbox.rs` rather than from prose:
 *
 *  - `observe_len` on a key the manifest does not grant **traps**. It does not
 *    return a sentinel. A guest that mistook denial for absence would report a
 *    confident verdict about a world it never saw.
 *  - `observe_len` on a granted-but-ungathered key returns `-1`.
 *  - `emit` accepts only 1/2/3; anything else traps.
 *  - Returning without emitting is *inconclusive*, not "holds".
 */

import { readFileSync } from 'node:fs';

export const HOLDS = 'holds';
export const FAILS = 'fails';
export const INDETERMINATE = 'indeterminate';
export const INCONCLUSIVE = 'inconclusive';

class Trap extends Error {}

export function runFalsifier(wasmPath, opts = {}) {
  const { inputs = {}, manifest = { observations: [] }, observations = {} } = opts;
  const bytes = readFileSync(wasmPath);
  const inputBytes = new TextEncoder().encode(JSON.stringify(inputs));

  let verdict = null;
  let message = '';
  let memory = null;
  const observed = [];

  const readStr = (ptr, len) =>
    new TextDecoder().decode(new Uint8Array(memory.buffer, ptr, len));
  const permits = (key) => (manifest.observations || []).includes(key);

  const imports = {
    crucible: {
      input_len: () => inputBytes.length,
      input_read: (ptr) => { new Uint8Array(memory.buffer).set(inputBytes, ptr); },
      observe_len: (keyPtr, keyLen) => {
        const key = readStr(keyPtr, keyLen);
        if (!permits(key)) throw new Trap('observation `' + key + '` is not granted by the manifest');
        if (!observed.includes(key)) observed.push(key);
        const v = observations[key];
        return v === undefined ? -1 : new TextEncoder().encode(v).length;
      },
      observe_read: (keyPtr, keyLen, outPtr) => {
        const key = readStr(keyPtr, keyLen);
        if (!permits(key)) throw new Trap('observation `' + key + '` is not granted by the manifest');
        const v = observations[key];
        if (v === undefined) throw new Trap('observation `' + key + '` was not gathered');
        new Uint8Array(memory.buffer).set(new TextEncoder().encode(v), outPtr);
      },
      emit: (v, ptr, len) => {
        const map = { 1: HOLDS, 2: FAILS, 3: INDETERMINATE };
        if (!map[v]) throw new Trap('emit: unknown verdict ' + v);
        verdict = map[v];
        message = readStr(ptr, len);
      },
    },
  };

  const inst = new WebAssembly.Instance(new WebAssembly.Module(bytes), imports);
  memory = inst.exports.memory;
  if (!memory) throw new Error(wasmPath + ' does not export memory');
  if (typeof inst.exports.crucible_falsify !== 'function') {
    throw new Error(wasmPath + ' does not export crucible_falsify');
  }

  try {
    inst.exports.crucible_falsify();
  } catch (e) {
    return { verdict: 'trapped', message: String(e.message || e), observed };
  }
  return { verdict: verdict === null ? INCONCLUSIVE : verdict, message, observed };
}

/**
 * The mutations `audit_vacuity` applies to a pure falsifier's inputs.
 * Ported from `crucible-probe/src/vacuity.rs::canonical_mutations`.
 */
export function canonicalMutations(inputs) {
  return [{}, structuralNegation(inputs), nullOut(inputs), '\u{1F480}crucible-vacuity-probe'];
}

function structuralNegation(v) {
  if (typeof v === 'boolean') return !v;
  if (typeof v === 'number') return -v;
  if (typeof v === 'string') return v === '' ? 'x' : '';
  if (Array.isArray(v)) return v.map(structuralNegation);
  if (v && typeof v === 'object') {
    return Object.fromEntries(Object.entries(v).map(function (e) {
      return [e[0], structuralNegation(e[1])];
    }));
  }
  return v;
}

function nullOut(v) {
  if (Array.isArray(v)) return v.map(function () { return null; });
  if (v && typeof v === 'object') {
    return Object.fromEntries(Object.keys(v).map(function (k) { return [k, null]; }));
  }
  return null;
}
