export async function register() {
  if (
    process.env.NEXT_RUNTIME === 'nodejs' &&
    process.env.NEXT_PHASE !== 'phase-production-build'
  ) {
    const { developmentScenario } = await import('./lib/development');
    if (developmentScenario()) return;
    const { webConfig } = await import('./lib/config');
    webConfig();
  }
}
