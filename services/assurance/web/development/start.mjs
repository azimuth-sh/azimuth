import { spawn } from 'node:child_process';

const scenario = process.argv[2] ?? 'mixed';
if (!['mixed', 'healthy', 'incident', 'empty'].includes(scenario))
  throw new Error('Choose mixed, healthy, incident or empty');
const port = process.env.ASSURANCE_UI_PORT ?? '3100';
if (!/^\d+$/.test(port) || Number(port) < 1024 || Number(port) > 65535)
  throw new Error('ASSURANCE_UI_PORT must be between 1024 and 65535');
const child = spawn(process.execPath, ['node_modules/next/dist/bin/next', 'dev', '--hostname', '127.0.0.1', '--port', port], {
  stdio: 'inherit', env: { ...process.env, NODE_ENV: 'development', ASSURANCE_UI_SCENARIO: scenario, NEXTAUTH_URL: `http://127.0.0.1:${port}` },
});
for (const signal of ['SIGINT', 'SIGTERM']) process.on(signal, () => child.kill(signal));
child.on('error', error => { console.error(error); process.exitCode = 1; });
child.on('exit', code => { process.exitCode = code ?? 1; });
