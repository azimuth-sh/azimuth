module.exports = {
  testMatch: '*.spec.cjs',
  workers: 1,
  retries: 2,
  webServer: {
    command: 'node ../server.cjs',
    port: Number(process.env.RETURNS_PORT),
    reuseExistingServer: false,
    gracefulShutdown: { signal: 'SIGTERM', timeout: 10000 },
    env: { SERVER_PORT: process.env.RETURNS_PORT },
  },
};
