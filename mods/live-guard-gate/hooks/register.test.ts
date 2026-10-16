import { test, expect, mock } from 'claude-code/testing'

const allow = () => ({ decision: 'allow' as const })
const check = ($: any, input: unknown) => $.tool.check({ tool: 'Bash', input })

test('terraform apply asks and names the IaC guard', async ($, on) => {
  mock.env(on, {})
  on('tool.check', allow)
  const v = await check($, { command: 'terraform apply -auto-approve' })
  expect(v.decision).toBe('ask')
  expect(v.reason).toContain('aws-live-iac-change-guard-agent')
})

test('az role assignment create asks and names the Entra guard', async ($, on) => {
  mock.env(on, {})
  on('tool.check', allow)
  const v = await check($, { command: 'az role assignment create --role Reader' })
  expect(v.decision).toBe('ask')
  expect(v.reason).toContain('azure-live-entra-role-assignment-guard-agent')
})

test('kubectl delete asks and names the rollout guard', async ($, on) => {
  mock.env(on, {})
  on('tool.check', allow)
  const v = await check($, { command: 'kubectl delete pod web-0' })
  expect(v.decision).toBe('ask')
  expect(v.reason).toContain('gcp-live-gke-rollout-guard-agent')
})

test('argv-array input asks', async ($, on) => {
  mock.env(on, {})
  on('tool.check', allow)
  const v = await check($, { argv: ['terraform', 'apply'] })
  expect(v.decision).toBe('ask')
})

test('read-only commands pass the engine verdict through', async ($, on) => {
  mock.env(on, {})
  on('tool.check', allow)
  for (const command of ['terraform plan', 'ls', 'az account show', 'kubectl get pods']) {
    expect((await check($, { command })).decision).toBe('allow')
  }
})

test('engine deny stays deny', async ($, on) => {
  mock.env(on, {})
  on('tool.check', () => ({ decision: 'deny' as const, reason: 'engine' }))
  const v = await check($, { command: 'terraform apply' })
  expect(v.decision).toBe('deny')
  expect(v.reason).toBe('engine')
})

test('oversized input asks', async ($, on) => {
  mock.env(on, {})
  on('tool.check', allow)
  const v = await check($, { command: 'a'.repeat(70000) })
  expect(v.decision).toBe('ask')
  expect(v.reason).toContain('input too large')
})
