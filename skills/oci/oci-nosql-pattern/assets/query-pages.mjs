/** Inject the real SDK query method; prepared is trusted/server-bound, not user SQL.
 * No SDK import, network, credentials or execution on module load. */
export async function* queryPages(query, prepared, {maxPages = 100, maxReadKB = 16, queryOptions = {}, signal} = {}) {
  for (const value of [maxPages, maxReadKB]) {
    if (!Number.isSafeInteger(value) || value <= 0) throw new RangeError("positive safe integer required");
  }
  if (queryOptions.continuationKey != null) throw new Error("helper starts fresh; external cursor unsupported");
  let continuationKey;
  for (let page = 0; page < maxPages; page++) {
    if (signal?.aborted) throw new Error("aborted");
    const options = {...queryOptions, maxReadKB};
    if (continuationKey != null) options.continuationKey = continuationKey;
    const result = await query(prepared, options);
    if (!result || !Array.isArray(result.rows)) throw new TypeError("invalid QueryResult");
    yield result; // Empty pages are valid; never infer completion from row count.
    continuationKey = result.continuationKey;
    if (continuationKey == null) return;
  }
  throw new Error("page budget exhausted; results incomplete");
}

/** Application contract, not native NDCS vector search or embedding generation. */
export function validateEmbedding({values, model, dimension, sourceId, sourceHash, tenantId}, expected) {
  if (!expected || typeof expected.tenantId !== "string" || !expected.tenantId.trim() ||
      typeof expected.model !== "string" || !expected.model.trim() ||
      tenantId !== expected.tenantId || model !== expected.model || dimension !== expected.dimension) {
    throw new Error("embedding space or tenant mismatch");
  }
  if (!Number.isSafeInteger(dimension) || dimension < 1 || !Array.isArray(values) || values.length !== dimension ||
      Array.from(values).some(x => typeof x !== "number" || !Number.isFinite(x))) throw new Error("invalid finite vector");
  if (typeof sourceId !== "string" || !sourceId.trim() || typeof sourceHash !== "string" || !sourceHash.trim()) throw new Error("missing source provenance");
  return {values: [...values], model, dimension, sourceId, sourceHash, tenantId};
}
