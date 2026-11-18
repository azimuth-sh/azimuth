module.exports = {
  testMatch: '*.spec.cjs',
  workers: 1,
  retries: 2,
  webServer: {
    command: 'node ../server.cjs',
    wait: { stdout: /DESK_READY (?<desk_port>\d+)/ },
    reuseExistingServer: false,
    gracefulShutdown: { signal: 'SIGTERM', timeout: 10000 },
  },
};
