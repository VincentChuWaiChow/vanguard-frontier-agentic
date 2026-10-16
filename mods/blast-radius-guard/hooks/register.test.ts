import { test, expect, mock } from 'claude-code/testing'

const ENV_MODE = 'VANGUARD_BLAST_RADIUS_GUARD_MODE'
const ENV_PATTERNS = 'VANGUARD_BLAST_RADIUS_GUARD_PATTERNS'
const allow = () => ({ decision: 'allow' as const })
const bash = (command: string) => ({ tool: 'Bash', input: { command } })
const ext = (over: object = {}) =>
  JSON.stringify([{ id: 'danger-cmd', name: 'danger cmd', regex: 'danger-cmd', flags: '', context: ['bash'], ...over }])

test('rm -rf / asks by default', async ($, on) => {
  mock.env(on, {})
  on('tool.check', allow)
  expect((await $.tool.check(bash('rm -rf /'))).decision).toBe('ask')
})

test('block mode denies', async ($, on) => {
  mock.env(on, { [ENV_MODE]: 'block' })
  on('tool.check', allow)
  expect((await $.tool.check(bash('rm -rf /'))).decision).toBe('deny')
})

test('unknown mode denies', async ($, on) => {
  mock.env(on, { [ENV_MODE]: 'bogus' })
  on('tool.check', allow)
  expect((await $.tool.check(bash('rm -rf /'))).decision).toBe('deny')
})

test('clean command passes the engine verdict', async ($, on) => {
  mock.env(on, {})
  on('tool.check', allow)
  expect((await $.tool.check(bash('ls'))).decision).toBe('allow')
})

test('engine deny stays deny', async ($, on) => {
  mock.env(on, {})
  on('tool.check', () => ({ decision: 'deny' as const, reason: 'engine' }))
  expect((await $.tool.check(bash('ls'))).decision).toBe('deny')
})

test('Write to a traversal path to /etc asks', async ($, on) => {
  mock.env(on, {})
  on('tool.check', allow)
  const v = await $.tool.check({ tool: 'Write', input: { file_path: '/tmp/../etc/passwd', content: 'x' } })
  expect(v.decision).toBe('ask')
})

test('Read of /etc/passwd is allowed', async ($, on) => {
  mock.env(on, {})
  on('tool.check', allow)
  const v = await $.tool.check({ tool: 'Read', input: { file_path: '/etc/passwd' } })
  expect(v.decision).toBe('allow')
})

test('argv array is read as a command line', async ($, on) => {
  mock.env(on, {})
  on('tool.check', allow)
  const v = await $.tool.check({ tool: 'Exec', input: { argv: ['rm', '-rf', '/'] } })
  expect(v.decision).toBe('ask')
})

test('extension pattern asks', async ($, on) => {
  mock.env(on, { [ENV_PATTERNS]: '/ext.json' })
  on('tool.check', allow)
  on('fs.read', () => ({ value: ext() }))
  expect((await $.tool.check(bash('run danger-cmd now'))).decision).toBe('ask')
})

test('unreadable extension file denies', async ($, on) => {
  mock.env(on, { [ENV_PATTERNS]: '/ext.json' })
  on('tool.check', allow)
  on('fs.read', () => {
    throw new Error('nope')
  })
  expect((await $.tool.check(bash('ls'))).decision).toBe('deny')
})

test('extension that is not JSON denies', async ($, on) => {
  mock.env(on, { [ENV_PATTERNS]: '/ext.json' })
  on('tool.check', allow)
  on('fs.read', () => ({ value: 'not json' }))
  expect((await $.tool.check(bash('ls'))).decision).toBe('deny')
})

test('extension reusing a built-in id denies', async ($, on) => {
  mock.env(on, { [ENV_PATTERNS]: '/ext.json' })
  on('tool.check', allow)
  on('fs.read', () => ({ value: ext({ id: 'rm-rf-root-or-home' }) }))
  expect((await $.tool.check(bash('ls'))).decision).toBe('deny')
})

test('oversized command asks as too large', async ($, on) => {
  mock.env(on, {})
  on('tool.check', allow)
  const v = await $.tool.check(bash('a'.repeat(70000)))
  expect(v.decision).toBe('ask')
  expect(v.reason).toContain('input too large to evaluate')
})
