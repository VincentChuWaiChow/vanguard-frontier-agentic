import test from "node:test";
import assert from "node:assert/strict";
import {readTenantPage, replaceOrder, retryRead429, vector, buildChunk} from "../assets/runtime.mjs";

function queryContainer(response) {
  const calls = [];
  return {calls, items: {
    query(spec, options) {
      calls.push({spec, options});
      return {async fetchNext() {return response;}};
    }
  }};
}

test("parameterized tenant + partition + opaque/session tokens and complete page", async () => {
  const c = queryContainer({resources: [{id:"1", tenantId:"tenant-a"}], requestCharge:4,
    continuationToken:"opaque-token", headers:{"x-ms-session-token":"session-v"}, activityId:"activity"});
  const page = await readTenantPage(c, {tenant:"tenant-a", sessionToken:"previous-session"});
  assert.equal(c.calls[0].options.partitionKey,"tenant-a");
  assert.deepEqual(c.calls[0].spec.parameters,[{name:"@tenant",value:"tenant-a"}]);
  assert.equal(c.calls[0].options.sessionToken,"previous-session");
  assert.equal(page.cursor.token,"opaque-token");
  assert.equal(page.sessionToken,"session-v");
  assert.equal(page.charge,4); assert.equal(page.overBudget,false);
});

test("empty page preserves continuation and resumes exactly that query scope", async () => {
  const c = queryContainer({resources:[],requestCharge:2,continuationToken:"next"});
  const page = await readTenantPage(c,{tenant:"a"});
  assert.deepEqual(page.items,[]); assert.ok(page.cursor);
  await readTenantPage(c,{tenant:"a",cursor:page.cursor});
  assert.equal(c.calls[1].options.continuationToken,"next");
  await assert.rejects(readTenantPage(c,{tenant:"b",cursor:page.cursor}),/scope mismatch/);
  assert.equal(c.calls.length,2);
});

test("tenant leak, malformed page, unknown RU and partial-page skipping fail", async () => {
  for(const response of [
    {resources:[{tenantId:"wrong"}],requestCharge:1},
    {resources:[{tenantId:"a"},{tenantId:"a"}],requestCharge:1},
    {resources:[],requestCharge:undefined}, {resources:[],requestCharge:NaN},
    {resources:null,requestCharge:1}, {resources:[],requestCharge:1,continuationToken:42}
  ]) await assert.rejects(readTenantPage(queryContainer(response),{tenant:"a",pageSize:1}));
});

test("positive page/RU caps reject invalid values before a client call", async () => {
  const c = queryContainer({resources:[],requestCharge:1});
  for(const args of [{tenant:""},{tenant:"a",pageSize:0},{tenant:"a",pageSize:101},
                    {tenant:"a",maxRU:0},{tenant:"a",pageSize:1.5}])
    await assert.rejects(readTenantPage(c,args));
  assert.equal(c.calls.length,0);
  const page = await readTenantPage(queryContainer({resources:[],requestCharge:12}),{tenant:"a",maxRU:10});
  assert.equal(page.overBudget,true); // caller stops; one-page cost known after response.
});

test("ETag replace pins id and tenant and doesn't retry 412", async () => {
  const calls=[];
  const c = {
    item(id, pk) {
      calls.push({id, pk});
      return {
        replace(body, options) {
          calls.push({body, options});
          return Promise.reject(Object.assign(new Error("stale"), {code:412}));
        }
      };
    }
  };
  await assert.rejects(replaceOrder(c,{tenant:"a",id:"1",body:{tenantId:"a",id:"1"},etag:'"v1"'}),{code:412});
  assert.equal(calls.length,2);
  assert.deepEqual(calls[1].options,{accessCondition:{type:"IfMatch",condition:'"v1"'}});
  for(const args of [{tenant:"a",id:"1",body:{tenantId:"b",id:"1"},etag:"v1"},
                    {tenant:"a",id:"1",body:{tenantId:"a",id:"1"},etag:""}])
    await assert.rejects(replaceOrder(c,args));
  assert.equal(calls.length,2);
});

test("429 honors server delay and cumulative attempt/deadline bounds", async () => {
  let now=0, attempts=0; const waits=[];
  const result=await retryRead429(async()=>{
    attempts++; if(attempts<3) throw {code:429,headers:{"x-ms-retry-after-ms":"10"}};
    return "ok";
  },{maxAttempts:3,deadlineMs:100,now:()=>now,sleep:async ms=>{waits.push(ms);now+=ms;}});
  assert.equal(result,"ok"); assert.equal(attempts,3); assert.deepEqual(waits,[10,10]);
});

test("read retry never retries auth/conflict/unknown writes and rejects missing delay", async () => {
  for(const error of [{code:403},{code:412},{code:500},{code:429,headers:{}},
                      {code:429,headers:{"x-ms-retry-after-ms":"100"}}]) {
    let attempts=0;
    await assert.rejects(retryRead429(async()=>{attempts++;throw error;},
      {maxAttempts:3,deadlineMs:50,now:()=>0,sleep:async()=>{throw new Error("must not sleep");}}));
    assert.equal(attempts,1);
  }
  let attempts=0;
  await assert.rejects(retryRead429(async()=>{attempts++;throw {code:429,headers:{"x-ms-retry-after-ms":"0"}};},
    {maxAttempts:2,deadlineMs:50,now:()=>0,sleep:async()=>{}}));
  assert.equal(attempts,2);
});

test("vector validates dimensions/type/finite/range and preserves float32 conversion", () => {
  assert.deepEqual(vector([1,2,3],{dimensions:3}),[1,2,3]);
  assert.equal(vector([0.1],{dimensions:1})[0],Math.fround(0.1));
  for(const values of [[1,NaN],[1,Infinity],[1,true],[1,"2"],[1,null],[0,0],[1,1e50],[1,1e-50],"source-url"])
    assert.throws(()=>vector(values,{dimensions:2}));
  for(const dimensions of [0,-1,1.5,4097]) assert.throws(()=>vector([1],{dimensions}));
  assert.throws(()=>vector(Array(506).fill(1),{dimensions:506,indexType:"flat"}));
  assert.throws(()=>vector([1],{dimensions:1,indexType:"fictional"}));
  assert.throws(()=>vector([1],{dimensions:1,dataType:"float16"}),/verify native float16/);
  assert.deepEqual(vector([-128,127],{dimensions:2,dataType:"int8"}),[-128,127]);
  assert.deepEqual(vector([0,255],{dimensions:2,dataType:"uint8"}),[0,255]);
  for(const [values,dataType] of [[[128],"int8"],[[-129],"int8"],[[-1],"uint8"],[[256],"uint8"],[[1.5],"int8"]])
    assert.throws(()=>vector(values,{dimensions:1,dataType}));
});

test("sparse vectors are rejected, never serialized into null coordinates", () => {
  assert.throws(()=>vector([1,,2],{dimensions:3}),/finite numeric/);
  assert.throws(()=>vector(new Array(3),{dimensions:3}),/finite numeric/);
  const trailing=[1]; trailing.length=2;
  assert.throws(()=>vector(trailing,{dimensions:2}),/finite numeric/);
});

const source={account:"syntheticblob",container:"docs",blob:"manual.pdf",versionId:"immutable-v1"};
test("embedding port is called only explicitly and result pins source/model/tenant", async () => {
  const calls=[]; const embed=async(text,options)=>{calls.push({text,options});return Array(1536).fill(0.1);};
  const result=await buildChunk(embed,{tenant:"a",id:"chunk1",text:"synthetic text",source,model:"model-generation-v1"});
  assert.equal(calls.length,1); assert.equal(calls[0].options.dimensions,1536);
  assert.equal(result.tenantId,"a"); assert.equal(result.source.versionId,"immutable-v1");
  assert.equal(result.embedding.length,1536); assert.equal(result.dataType,"float32");
});

test("bad source or empty context fails before embedding; dimensions/model output verified", async () => {
  let calls=0; const embed=async()=>{calls++;return [1,2];};
  for(const change of [{source:{...source,versionId:"null"}}, {source:{...source,blob:"x?sig=secret"}},
                      {source:{account:"x",container:"x",blob:"x"}}, {tenant:""}, {model:""}])
    await assert.rejects(buildChunk(embed,{tenant:"a",id:"1",text:"text",source,model:"v1",...change}));
  assert.equal(calls,0);
  await assert.rejects(buildChunk(embed,{tenant:"a",id:"1",text:"text",source,model:"v1"}),/dimension/);
  assert.equal(calls,1);
});
