/** Injected Cosmos Container/example ports only. Importing makes no calls. */
const QUERY_ID = "tenant-orders-v1";
function required(...values) {
  if (values.some(v => typeof v !== "string" || !v.trim())) throw new Error("explicit nonempty scope required");
}
function cap(value, max = Number.MAX_SAFE_INTEGER) {
  if (!Number.isSafeInteger(value) || value < 1 || value > max) throw new Error("positive bounded integer required");
}

export async function readTenantPage(container, {tenant, cursor = null, sessionToken,
  pageSize = 50, maxRU = 1000}) {
  required(tenant); cap(pageSize, 100); cap(maxRU);
  if (cursor && (cursor.tenant !== tenant || cursor.queryId !== QUERY_ID ||
      typeof cursor.token !== "string" || !cursor.token)) throw new Error("cursor scope mismatch");
  if (sessionToken !== undefined) required(sessionToken);
  // Cursor authenticity/signing, authenticated tenant and endpoint/container
  // selection belong to the trusted server; these checks are defense in depth.
  const iterator = container.items.query({
    query: "SELECT * FROM c WHERE c.tenantId=@tenant ORDER BY c.createdAt DESC,c.id ASC",
    parameters: [{name: "@tenant", value: tenant}]
  }, {partitionKey: tenant, maxItemCount: pageSize,
      ...(cursor ? {continuationToken: cursor.token} : {}),
      ...(sessionToken ? {sessionToken} : {})});
  const response = await iterator.fetchNext();
  if (!Array.isArray(response.resources) || response.resources.length > pageSize)
    throw new Error("unexpected service page; don't truncate and skip with end cursor");
  if (response.resources.some(row => row.tenantId !== tenant)) throw new Error("tenant isolation violation");
  const charge = response.requestCharge;
  if (typeof charge !== "number" || !Number.isFinite(charge) || charge < 0)
    throw new Error("unknown RU charge; stop rather than assume zero");
  const token = response.continuationToken;
  if (token !== undefined && token !== null && typeof token !== "string") throw new Error("invalid continuation");
  return {items: response.resources, charge, overBudget: charge > maxRU,
    cursor: token ? {token, tenant, queryId: QUERY_ID} : null,
    sessionToken: response.headers?.["x-ms-session-token"],
    activityId: response.activityId}; // Never log tokens/items by default.
}

export async function replaceOrder(container, {tenant, id, body, etag}) {
  required(tenant, id, etag);
  if (!body || body.id !== id || body.tenantId !== tenant) throw new Error("write scope mismatch");
  // No generic retry: 412 is conflict; ambiguous writes require reconciliation.
  return container.item(id, tenant).replace(body,
    {accessCondition: {type: "IfMatch", condition: etag}});
}

export async function retryRead429(read, {maxAttempts, deadlineMs, now, sleep}) {
  cap(maxAttempts); cap(deadlineMs);
  const end = now() + deadlineMs;
  for (let attempt = 1; attempt <= maxAttempts; attempt++) {
    if (now() >= end) throw new Error("read deadline exhausted");
    try { return await read(); }
    catch (error) {
      if (error.code !== 429 || attempt === maxAttempts) throw error;
      const delay = Number(error.headers?.["x-ms-retry-after-ms"]);
      if (!Number.isFinite(delay) || delay < 0 || now() + delay >= end)
        throw new Error("retry delay missing/invalid/exceeds deadline");
      await sleep(delay);
    }
  }
  throw new Error("unreachable");
}

export function vector(values, {dimensions, dataType = "float32", indexType = "diskANN"}) {
  cap(dimensions, indexType === "flat" ? 505 : 4096);
  if (!["flat", "quantizedFlat", "diskANN"].includes(indexType)) throw new Error("unsupported index");
  if (!["float32", "int8", "uint8"].includes(dataType))
    throw new Error("example conversion only supports float32/int8/uint8; verify native float16 separately");
  if (!Array.isArray(values) || values.length !== dimensions) throw new Error("vector dimension mismatch");
  // Array.from visits holes as undefined; Array.map alone would skip them and
  // JSON serialization would silently turn sparse coordinates into null.
  const converted = Array.from(values, value => {
    if (typeof value !== "number" || !Number.isFinite(value)) throw new Error("finite numeric coordinate required");
    if (dataType === "float32") {
      const rounded = Math.fround(value);
      if (!Number.isFinite(rounded) || (value !== 0 && rounded === 0)) throw new Error("float32 overflow/underflow");
      return rounded;
    }
    const [min, max] = dataType === "int8" ? [-128, 127] : [0, 255];
    if (!Number.isInteger(value) || value < min || value > max) throw new Error("integer vector range violation");
    return value;
  });
  if (!converted.some(x => x !== 0)) throw new Error("zero vector unsuitable for this cosine example");
  return converted;
}

export async function buildChunk(embed, {tenant, id, text, source, model}) {
  required(tenant, id, text, model);
  if (!source || Object.keys(source).sort().join(",") !== "account,blob,container,versionId")
    throw new Error("pinned Blob source contract required");
  required(...Object.values(source));
  if (source.versionId === "null" || Object.values(source).some(v => /[?&]sig=/i.test(v)))
    throw new Error("immutable version required; no persisted SAS secrets");
  // embed is a caller-supplied PORT, not a claimed Azure SDK method. Caller owns
  // explicit model-call authorization, token budget, deadline and actual adapter.
  const coords = vector(await embed(text, {model, dimensions: 1536}), {dimensions: 1536});
  return {id, tenantId: tenant, text, source: {...source}, embedding: coords,
    model, dimensions: 1536, metric: "cosine", dataType: "float32", status: "published"};
}
