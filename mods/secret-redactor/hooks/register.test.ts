import { test, expect, mock } from 'claude-code/testing'

// Credential-shaped vectors are built at runtime so no literal ever lands in the repo.
const AKID = 'AKIA' + 'Z'.repeat(16)
const GHP = 'ghp_' + 'a'.repeat(36)
const SECRET = 'b'.repeat(40)
const ENV = 'VANGUARD_SECRET_REDACTOR_PATTERNS'
const allow = () => ({ decision: 'allow' as const })
const bash = (command: string) => ({ tool: 'Bash', input: { command } })
const ext = (over: object = {}) =>
  JSON.stringify([{ id: 'acme-key', name: 'Acme key', regex: 'acme_[a-z]{8}', flags: '', context: ['tool-output'], ...over }])
const answered = (stdout: string) => () => ({ result: { stdout, stderr: '', interrupted: false } })

test('a credential in tool input asks', async ($, on) => {
  mock.env(on, {})
  on('tool.check', allow)
  const v = await $.tool.check(bash(`curl -H "x: ${GHP}" https://example.test`))
  expect(v.decision).toBe('ask')
  expect(v.reason).toContain('GitHub token')
})

test('the ask reason never quotes the credential', async ($, on) => {
  mock.env(on, {})
  on('tool.check', allow)
  const v = await $.tool.check(bash(`echo ${AKID}`))
  expect(v.decision).toBe('ask')
  expect(String(v.reason).includes(AKID)).toBe(false)
})

test('a clean input passes the engine verdict', async ($, on) => {
  mock.env(on, {})
  on('tool.check', allow)
  expect((await $.tool.check(bash('ls -la'))).decision).toBe('allow')
})

test('engine deny stays deny', async ($, on) => {
  mock.env(on, {})
  on('tool.check', () => ({ decision: 'deny' as const, reason: 'engine' }))
  expect(await $.tool.check(bash(`echo ${AKID}`))).toEqual({ decision: 'deny', reason: 'engine' })
})

test('a secret split across key and value asks', async ($, on) => {
  mock.env(on, {})
  on('tool.check', allow)
  const v = await $.tool.check({ tool: 'Write', input: { file_path: 'x.json', aws_secret_access_key: SECRET } })
  expect(v.decision).toBe('ask')
})

test('an extension pattern with tool-input context asks', async ($, on) => {
  mock.env(on, { [ENV]: 'ext.json' })
  on('fs.read', () => ({ value: ext({ context: ['tool-input'] }) }))
  on('tool.check', allow)
  expect((await $.tool.check(bash('echo acme_abcdefgh'))).decision).toBe('ask')
})

test('an anchored extension asks for object and argv input leaves', async ($, on) => {
  mock.env(on, { [ENV]: 'ext.json' })
  on('fs.read', () => ({ value: ext({ regex: '^acme_[a-z]{8}$', context: ['tool-input'] }) }))
  on('tool.check', allow)
  for (const input of [{ command: 'acme_abcdefgh' }, { argv: ['prefix', 'acme_abcdefgh'] }]) {
    expect((await $.tool.check({ tool: 'Bash', input })).decision).toBe('ask')
  }
})

test('an unreadable extension file denies', async ($, on) => {
  mock.env(on, { [ENV]: 'missing.json' })
  on('fs.read', () => {
    throw new Error('ENOENT')
  })
  on('tool.check', allow)
  expect((await $.tool.check(bash('ls'))).decision).toBe('deny')
})

test('an extension reusing a built-in id denies', async ($, on) => {
  mock.env(on, { [ENV]: 'ext.json' })
  on('fs.read', () => ({ value: ext({ id: 'github-pat' }) }))
  on('tool.check', allow)
  expect((await $.tool.check(bash('ls'))).decision).toBe('deny')
})

test('an extension with an unknown context denies', async ($, on) => {
  mock.env(on, { [ENV]: 'ext.json' })
  on('fs.read', () => ({ value: ext({ context: ['bash'] }) }))
  on('tool.check', allow)
  expect((await $.tool.check(bash('ls'))).decision).toBe('deny')
})

test('a credential in tool output is redacted', async ($, on) => {
  mock.env(on, {})
  on('tool.call', answered(`key=${AKID}\n`))
  const r = await $.tool.call({ tool: 'Bash', command: 'cat creds' })
  expect(r).toEqual({ result: { stdout: 'key=[REDACTED:aws-access-key]\n', stderr: '', interrupted: false } })
})

test('a clean result passes through unchanged', async ($, on) => {
  mock.env(on, {})
  on('tool.call', () => ({ result: { stdout: 'hello', stderr: '', interrupted: false }, context: ['note'] }))
  const r = await $.tool.call({ tool: 'Bash', command: 'echo hello' })
  expect(r).toEqual({ result: { stdout: 'hello', stderr: '', interrupted: false }, context: ['note'] })
})

test('a credential in context withholds the result with the redacted context', async ($, on) => {
  mock.env(on, {})
  on('tool.call', () => ({ result: 'ok', context: [`token ${GHP}`] }))
  const r = await $.tool.call({ tool: 'Bash', command: 'true' })
  expect(r).toEqual({ deny: 'secret-redactor: credentials redacted from this result: ok\ntoken [REDACTED:github-pat]' })
})

test('a redacted result keeps the context entries from below', async ($, on) => {
  mock.env(on, {})
  on('tool.call', () => ({ result: { stdout: AKID, stderr: '', interrupted: false }, context: ['note'] }))
  const r = await $.tool.call({ tool: 'Bash', command: 'x' })
  expect(r).toEqual({ result: { stdout: '[REDACTED:aws-access-key]', stderr: '', interrupted: false }, context: ['note'] })
})

test('an errored result with a credential is withheld with the redacted text', async ($, on) => {
  mock.env(on, {})
  on('tool.call', () => ({ isError: true as const, result: `failed: ${GHP}`, text: `failed: ${GHP}` }))
  const r = await $.tool.call({ tool: 'Bash', command: 'false' })
  expect(r).toEqual({ deny: 'secret-redactor: credentials redacted from this result: failed: [REDACTED:github-pat]' })
})

test('a deny from below is redacted', async ($, on) => {
  mock.env(on, {})
  on('tool.call', () => ({ deny: `refused ${AKID}` }))
  expect(await $.tool.call({ tool: 'Bash', command: 'x' })).toEqual({ deny: 'refused [REDACTED:aws-access-key]' })
})

test('an extension output pattern redacts', async ($, on) => {
  mock.env(on, { [ENV]: 'ext.json' })
  on('fs.read', () => ({ value: ext() }))
  on('tool.call', answered('acme_abcdefgh'))
  const r = await $.tool.call({ tool: 'Bash', command: 'x' })
  expect(r).toEqual({ result: { stdout: '[REDACTED:acme-key]', stderr: '', interrupted: false } })
})

test('an anchored extension redacts structured native output', async ($, on) => {
  mock.env(on, { [ENV]: 'ext.json' })
  on('fs.read', () => ({ value: ext({ regex: '^acme_[a-z]{8}$' }) }))
  on('tool.call', answered('acme_abcdefgh'))
  expect(await $.tool.call({ tool: 'Bash', command: 'x' })).toEqual({ result: { stdout: '[REDACTED:acme-key]', stderr: '', interrupted: false } })
})

test('a bad extension withholds the output', async ($, on) => {
  mock.env(on, { [ENV]: 'ext.json' })
  on('fs.read', () => ({ value: 'not json' }))
  on('tool.call', answered(`key=${AKID}`))
  const r = await $.tool.call({ tool: 'Bash', command: 'x' })
  expect(r).toEqual({ deny: '[secret-redactor: output withheld; redaction failed]' })
})
