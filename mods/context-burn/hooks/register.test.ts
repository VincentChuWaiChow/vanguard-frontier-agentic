import { test, expect, mock } from 'claude-code/testing'

const ENV = 'VANGUARD_CONTEXT_BURN_STATUS_PATH'
const DEFAULT = '.claude/context-burn-status/e8bc163c82eee18733288c7d4ac636db3a6deb013ef2d37b68322be20edc45cc.json'
const RESULT = { result: { stdout: 'x', stderr: '', interrupted: false } }
const call = { tool: 'Bash', command: 'echo x' }
const status = (over: object = {}) => ({
  session_id: 's1',
  tool_call_count: 4,
  last_updated: '1970-01-01T00:00:00.000Z',
  threshold_events: [],
  ...over,
})

// Wires every `$` noun a tool.call touches. Returns what the mod wrote and toasted.
function rig(on: any, o: { env?: object; prev?: object | null; readFails?: boolean; writeFails?: boolean; context?: object; session?: { id: string }; files?: Map<string, string> } = {}) {
  const writes: { path: string; text: string }[] = []
  const toasts: string[] = []
  mock.env(on, o.env ?? {})
  mock.clock(on, { now: 0 })
  on('session.id', () => ({ value: o.session?.id ?? 's1' }))
  on('session.usage', () => ({
    value: { startedAt: 0, context: o.context ?? { window: 10000 }, rateLimits: [] },
  }))
  on('fs.read', ($: any, e: any) => {
    if (o.files) {
      if (!o.files.has(e.path)) throw new Error('ENOENT')
      return { value: o.files.get(e.path) }
    }
    if (o.readFails || !o.prev) throw new Error('ENOENT')
    return { value: JSON.stringify(o.prev) }
  })
  on('fs.write', ($: any, e: any) => {
    if (o.writeFails) throw new Error('disk full')
    writes.push({ path: e.path, text: e.text })
    o.files?.set(e.path, e.text)
    return { value: undefined }
  })
  on('ui.toast', ($: any, e: any) => {
    toasts.push(e.text)
    return { value: undefined }
  })
  on('tool.call', () => RESULT)
  on('session.start', ($: any, e: any) => ({ cwd: e.cwd }))
  return { writes, toasts, last: () => JSON.parse(writes[writes.length - 1].text) }
}

test('session.start writes a fresh status', async ($, on) => {
  const r = rig(on)
  await $.session.start({ cwd: '/w', surface: 'terminal', isInteractive: true })
  expect(r.writes[0].path.endsWith(DEFAULT)).toBe(true)
  expect(r.last()).toEqual({ session_id: 's1', tool_call_count: 0, last_updated: '1970-01-01T00:00:00.000Z', threshold_events: [] })
})

test('session.start honors the env override', async ($, on) => {
  const r = rig(on, { env: { [ENV]: 'custom/out.json' } })
  await $.session.start({ cwd: '/w', surface: 'terminal', isInteractive: true })
  expect(r.writes[0].path.endsWith('custom/out.json')).toBe(true)
})

test('tool.call passes the result through and increments the count', async ($, on) => {
  const r = rig(on, { prev: status() })
  expect(await $.tool.call(call)).toEqual(RESULT)
  expect(r.last().tool_call_count).toBe(5)
})

test('a status for another session restarts at 1', async ($, on) => {
  const r = rig(on, { prev: status({ session_id: 'other' }) })
  await $.tool.call(call)
  expect(r.last().session_id).toBe('s1')
  expect(r.last().tool_call_count).toBe(1)
})

test('75 percent toasts one warning and records the event', async ($, on) => {
  const r = rig(on, { prev: status(), context: { tokens: 7500, window: 10000, percent: 75 } })
  await $.tool.call(call)
  expect(r.toasts).toEqual(['[context-burn] WARNING: 70% context used (7500 / 10000 tokens)'])
  expect(r.last().threshold_events.length).toBe(1)
  expect(r.last().threshold_events[0].threshold).toBe(70)
})

test('an already recorded 70 event does not toast again', async ($, on) => {
  const prev = status({ threshold_events: [{ threshold: 70, tokens: 7500, window: 10000, at: 'x' }] })
  const r = rig(on, { prev, context: { tokens: 7600, window: 10000, percent: 76 } })
  await $.tool.call(call)
  expect(r.toasts).toEqual([])
})

test('95 percent toasts warning then critical', async ($, on) => {
  const r = rig(on, { prev: status(), context: { tokens: 9500, window: 10000, percent: 95 } })
  await $.tool.call(call)
  expect(r.toasts).toEqual([
    '[context-burn] WARNING: 70% context used (9500 / 10000 tokens)',
    '[context-burn] CRITICAL: 90% context used (9500 / 10000 tokens)',
  ])
})

test('percent without tokens computes and rounds tokens', async ($, on) => {
  const r = rig(on, { prev: status(), context: { window: 3333, percent: 71.5 } })
  await $.tool.call(call)
  expect(r.toasts).toEqual(['[context-burn] WARNING: 70% context used (2383 / 3333 tokens)'])
})

test('no token data means no toast but the count still rises', async ($, on) => {
  const r = rig(on, { prev: status(), context: { window: 10000 } })
  await $.tool.call(call)
  expect(r.toasts).toEqual([])
  expect(r.last().tool_call_count).toBe(5)
})

test('a failing read starts a fresh count', async ($, on) => {
  const r = rig(on, { readFails: true })
  await $.tool.call(call)
  expect(r.last().tool_call_count).toBe(1)
})

test('malformed threshold events restart counting and still toast the threshold', async ($, on) => {
  const r = rig(on, { prev: status({ threshold_events: [null] }), context: { tokens: 7500, window: 10000 } })
  expect(await $.tool.call(call)).toEqual(RESULT)
  expect(r.last().tool_call_count).toBe(1)
  expect(r.last().threshold_events.length).toBe(1)
  expect(r.toasts.length).toBe(1)
})

test('interleaved sessions retain independent counts and warn only once per session', async ($, on) => {
  const session = { id: '../first' }
  const files = new Map<string, string>()
  const r = rig(on, { session, files, context: { tokens: 7500, window: 10000 } })
  for (const id of ['../first', '..\\second']) {
    session.id = id
    await $.session.start({ cwd: '/w', surface: 'terminal', isInteractive: true })
    expect(await $.tool.call(call)).toEqual(RESULT)
  }
  for (const id of ['../first', '..\\second']) {
    session.id = id
    expect(await $.tool.call(call)).toEqual(RESULT)
  }
  expect(files.size).toBe(2)
  for (const [path, text] of files) {
    expect(/\/context-burn-status\/[a-f0-9]{64}\.json$/.test(path)).toBe(true)
    const saved = JSON.parse(text)
    expect(saved.tool_call_count).toBe(2)
    expect(saved.threshold_events.length).toBe(1)
  }
  expect(r.toasts.length).toBe(2)
})

test('a failing write still returns the tool result', async ($, on) => {
  rig(on, { prev: status(), writeFails: true })
  expect(await $.tool.call(call)).toEqual(RESULT)
})
