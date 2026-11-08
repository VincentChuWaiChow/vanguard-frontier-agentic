/**
 * Internal, trusted request input only. query(request) is an injected async SDK
 * adapter. No SDK dependency, network activity or credential lookup on import.
 * Yield whole pages, including empty ones, so callers retain continuation state.
 * This does not validate public cursors, authorize tenants or create snapshots.
 */
export async function* queryPages(query, input, { maxPages = 100 } = {}) {
  if (!Number.isInteger(maxPages) || maxPages < 1) {
    throw new RangeError("maxPages must be a positive integer");
  }
  let cursor = input.ExclusiveStartKey;
  for (let page = 0; page < maxPages; page += 1) {
    const request = { ...input };
    // Snapshot the request's continuation reference; never mutate caller input.
    if (cursor && Object.keys(cursor).length > 0) {
      request.ExclusiveStartKey = cursor;
    } else {
      delete request.ExclusiveStartKey;
    }
    const result = await query(request);
    yield result;
    cursor = result.LastEvaluatedKey;
    if (!cursor || Object.keys(cursor).length === 0) return;
  }
  throw new Error("Query page budget exhausted; results are incomplete");
}

/**
 * Bounded HTTP match-page collector for an internal, authorized Query request.
 * Return the service continuation even after empty/short pages or budget expiry.
 * The API layer must sign/store it and enforce tenant/query binding externally.
 */
export async function matchingPage(query, input, { target = 50, maxPages = 10 } = {}) {
  const cap = input.Limit ?? 50;
  for (const value of [target, maxPages, cap]) {
    if (!Number.isInteger(value) || value < 1) {
      throw new RangeError("target, maxPages and Limit must be positive integers");
    }
  }
  const Items = [];
  let cursor = input.ExclusiveStartKey;
  for (let page = 0; page < maxPages; page += 1) {
    const request = { ...input, Limit: Math.min(cap, target - Items.length) };
    if (cursor && Object.keys(cursor).length > 0) request.ExclusiveStartKey = cursor;
    else delete request.ExclusiveStartKey;
    const result = await query(request);
    // Returned matches <= evaluated items <= requested Limit: no slicing needed.
    Items.push(...(result.Items ?? []));
    cursor = result.LastEvaluatedKey;
    if (!cursor || Object.keys(cursor).length === 0) return { Items };
    if (Items.length === target) return { Items, LastEvaluatedKey: cursor };
  }
  return { Items, LastEvaluatedKey: cursor, WorkBudgetExhausted: true };
}
