import assert from 'node:assert/strict';
import fc from 'fast-check';

const nullPrototype = Object.assign(Object.create(null), { ok: 1 });
assert.equal(JSON.stringify(nullPrototype), '{"ok":1}');
const unsafe = { missing: undefined, infinite: Infinity };
assert.notDeepEqual(JSON.parse(JSON.stringify(unsafe)), unsafe);
assert.throws(() => JSON.stringify({ bigint: 1n }), TypeError);
fc.assert(fc.property(fc.jsonValue(), value => {
  const json = JSON.stringify(value);
  assert.equal(typeof json, 'string');
  assert.doesNotThrow(() => JSON.parse(json));
}), { numRuns: 200, seed: 20261015 });
console.log(`PASS: fast-check ${fc.__version}; null prototype serializes; non-JSON counterexamples rejected; 200 seeded JSON-value probes`);
