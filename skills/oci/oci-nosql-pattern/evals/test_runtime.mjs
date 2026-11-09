import test from 'node:test';
import assert from 'node:assert/strict';
import {queryPages, validateEmbedding} from '../assets/query-pages.mjs';

async function collect(iterator) {
  const rows = [];
  for await (const page of iterator) rows.push(...page.rows);
  return rows;
}

test('actual helper follows empty page continuation and preserves exact opaque token/options', async () => {
  const token = Uint8Array.from([1, 255, 0]);
  const prepared = Object.freeze({serverBoundTenant: 'tenant-A'});
  const queryOptions = Object.freeze({compartment: 'compartment-A', consistency: 'ABSOLUTE'});
  const calls = [];
  const results = [{rows: [], continuationKey: token}, {rows: [{orderId:'one'}]}];
  const query = async (stmt, options) => { calls.push([stmt, options]); return results.shift(); };
  assert.deepEqual(await collect(queryPages(query, prepared, {queryOptions, maxPages:2, maxReadKB:16})), [{orderId:'one'}]);
  assert.equal(calls.length, 2);
  assert.equal(calls[0][0], prepared);
  assert.equal(calls[0][1].compartment, 'compartment-A');
  assert.equal(calls[0][1].consistency, 'ABSOLUTE');
  assert.equal(calls[1][1].continuationKey, token);
  assert.equal(calls[0][1].continuationKey, undefined);
  assert.equal(queryOptions.continuationKey, undefined);
});

test('page budget fails incomplete without unbounded loop or silently succeeding', async () => {
  let calls=0;
  const query=async()=>{calls++; return {rows:[], continuationKey: 'repeat'};};
  await assert.rejects(collect(queryPages(query, {}, {maxPages:3})), /incomplete/);
  assert.equal(calls,3);
});

test('invalid bounds, external cursor and abort fail before SDK call', async () => {
  let calls=0;
  const query=async()=>{calls++; return {rows:[]};};
  for(const options of [{maxPages:0},{maxPages:1.5},{maxReadKB:-1},{maxReadKB:NaN},{maxPages:Infinity},
      {queryOptions:{continuationKey:'public-untrusted'}},{signal:{aborted:true}}]) {
    await assert.rejects(collect(queryPages(query, {}, options)));
  }
  assert.equal(calls,0);
});

test('SDK errors and malformed pages propagate; no hidden retries or partial-success claims', async () => {
  let calls=0;
  await assert.rejects(collect(queryPages(async()=>{calls++; throw new Error('Forbidden');},{})), /Forbidden/);
  assert.equal(calls,1);
  await assert.rejects(collect(queryPages(async()=>({rows:null}),{})), /QueryResult/);
});

const expected={tenantId:'tenant-A',model:'model-v1',dimension:3};
const valid={values:[0.1,-0.2,0.3],tenantId:'tenant-A',model:'model-v1',dimension:3,sourceId:'doc-1',sourceHash:'hash-1'};

test('embedding contract preserves numeric coordinates, tenant/model and source provenance', () => {
  const output=validateEmbedding(valid,expected);
  assert.deepEqual(output,valid);
  assert.notEqual(output.values,valid.values);
  output.values[0]=8;
  assert.equal(valid.values[0],0.1);
});

test('embedding rejects source URI instead of coordinates, nonfinite values and wrong dimensions/spaces/tenant', () => {
  const mutations=[{values:'oci://bucket/source'},{values:[0.1,0.2]},{values:[0.1,NaN,0.3]},
    {values:[0.1,Infinity,0.3]},{values:[0.1,'0.2',0.3]},{dimension:0},{dimension:1.5},
    {tenantId:'tenant-B'},{model:'model-v2'},{sourceId:''},{sourceHash:''},{sourceHash:' '}];
  for(const change of mutations) assert.throws(()=>validateEmbedding({...valid,...change},expected));
});

test('sparse coordinates and sparse source metadata are not finite vectors', () => {
  const sparse=Array(3); sparse[0]=1;
  assert.throws(()=>validateEmbedding({...valid,values:sparse},expected));
});

test('embedding requires an explicit nonempty trusted expected tenant/model', () => {
  assert.throws(()=>validateEmbedding({...valid,tenantId:undefined,model:undefined}, {dimension:3}));
});
