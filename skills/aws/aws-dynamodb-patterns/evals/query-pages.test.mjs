import test from "node:test";
import assert from "node:assert/strict";
import { matchingPage, queryPages } from "../assets/query-pages.mjs";

const input = {
  TableName: "OfflineOnly",
  KeyConditionExpression: "#pk = :pk",
  ExpressionAttributeNames: { "#pk": "PK" },
  ExpressionAttributeValues: { ":pk": { S: "TENANT#test" } },
  Limit: 50,
};
const key = { PK: { S: "TENANT#test" }, SK: { S: "ORDER#1" } };

async function collect(generator) {
  const pages = [];
  for await (const page of generator) pages.push(page);
  return pages;
}

test("empty filtered page continues using the exact typed key", async () => {
  const original = structuredClone(input);
  const calls = [];
  const responses = [
    { Items: [], LastEvaluatedKey: key },
    { Items: [{ SK: { S: "ORDER#2" } }], LastEvaluatedKey: {} },
  ];
  const pages = await collect(queryPages(async request => {
    calls.push(request);
    return responses[calls.length - 1];
  }, input));
  assert.equal(pages.length, 2);
  assert.equal(pages[1].Items[0].SK.S, "ORDER#2");
  assert.equal(calls[1].ExclusiveStartKey, key);
  assert.equal("ExclusiveStartKey" in calls[0], false);
  assert.deepEqual(input, original);
  assert.equal(calls[1].Limit, 50);
  assert.equal(calls[1].KeyConditionExpression, input.KeyConditionExpression);
});

for (const terminal of [undefined, null, {}]) {
  test(`absent/null/empty terminal cursor ends (${JSON.stringify(terminal)})`, async () => {
    let calls = 0;
    const pages = await collect(queryPages(async () => {
      calls += 1;
      return { Items: [], LastEvaluatedKey: terminal };
    }, input));
    assert.equal(calls, 1);
    assert.equal(pages.length, 1);
  });
}

test("nonempty initial cursor is forwarded without rewriting numbers/binary", async () => {
  const cursor = { PK: { N: "9007199254740993" }, SK: { B: new Uint8Array([1, 2]) } };
  await collect(queryPages(async request => {
    assert.equal(request.ExclusiveStartKey, cursor);
    return {};
  }, { ...input, ExclusiveStartKey: cursor }));
});

test("empty initial cursor is omitted", async () => {
  await collect(queryPages(async request => {
    assert.equal("ExclusiveStartKey" in request, false);
    return {};
  }, { ...input, ExclusiveStartKey: {} }));
});

test("page budget reports incompleteness and makes no extra call", async () => {
  let calls = 0;
  await assert.rejects(collect(queryPages(async () => {
    calls += 1;
    return { Items: [], LastEvaluatedKey: key };
  }, input, { maxPages: 2 })), /budget exhausted.*incomplete/);
  assert.equal(calls, 2);
});

test("invalid budgets fail before any SDK call", async () => {
  for (const maxPages of [0, -1, 1.5, NaN, Infinity]) {
    await assert.rejects(collect(queryPages(async () => {
      assert.fail("must not query");
    }, input, { maxPages })), RangeError);
  }
});

test("SDK errors propagate; helper does not silently retry unknown outcomes", async () => {
  const error = new Error("timeout");
  let calls = 0;
  await assert.rejects(collect(queryPages(async () => {
    calls += 1;
    throw error;
  }, input)), failure => failure === error);
  assert.equal(calls, 1);
});

test("caller break stops lazy pagination without another SDK request", async () => {
  let calls = 0;
  for await (const page of queryPages(async () => {
    calls += 1;
    return { Items: [], LastEvaluatedKey: key };
  }, input)) {
    assert.ok(page.LastEvaluatedKey);
    break;
  }
  assert.equal(calls, 1);
});

test("match page uses remaining count, preserving surplus candidates for next page", async () => {
  const all = Array.from({ length: 80 }, (_, n) => ({ id: n }));
  const limits = [];
  // First call evaluates 50 but only 40 match; later calls match every item.
  const query = async request => {
    limits.push(request.Limit);
    const start = request.ExclusiveStartKey?.offset ?? 0;
    const end = Math.min(start + request.Limit, all.length);
    return {
      Items: all.slice(start, end).filter(item => item.id >= 10),
      LastEvaluatedKey: end < all.length ? { offset: end } : {},
    };
  };
  const first = await matchingPage(query, input);
  assert.deepEqual(limits, [50, 10]);
  assert.equal(first.Items.length, 50);
  assert.deepEqual(first.LastEvaluatedKey, { offset: 60 });
  const second = await matchingPage(query, { ...input, ExclusiveStartKey: first.LastEvaluatedKey });
  assert.equal(second.Items.length, 20);
  const returnedIds = [...first.Items, ...second.Items].map(item => item.id);
  assert.deepEqual(returnedIds, all.slice(10).map(item => item.id));
});

test("match page continues after empty page and returns budget cursor", async () => {
  let calls = 0;
  const response = await matchingPage(async () => {
    calls += 1;
    return { Items: [], LastEvaluatedKey: key };
  }, input, { maxPages: 2 });
  assert.equal(calls, 2);
  assert.deepEqual(response.Items, []);
  assert.equal(response.LastEvaluatedKey, key);
  assert.equal(response.WorkBudgetExhausted, true);
});

test("match page handles absent/empty final keys without leaving stale input cursor", async () => {
  for (const terminal of [undefined, {}]) {
    const response = await matchingPage(async request => {
      assert.equal("ExclusiveStartKey" in request, false);
      return { Items: [{ id: 1 }], LastEvaluatedKey: terminal };
    }, { ...input, ExclusiveStartKey: {} });
    assert.deepEqual(response, { Items: [{ id: 1 }] });
  }
});

test("match page rejects invalid work bounds before querying", async () => {
  for (const options of [{ target: 0 }, { target: 1.5 }, { maxPages: -1 }]) {
    await assert.rejects(matchingPage(async () => assert.fail(), input, options), RangeError);
  }
  await assert.rejects(matchingPage(async () => assert.fail(), { ...input, Limit: 0 }), RangeError);
});
