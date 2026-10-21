import 'server-only';

export function developmentScenario(): string | null {
  const scenario = process.env.ASSURANCE_UI_SCENARIO;
  if (!scenario) return null;
  if (process.env.NODE_ENV !== 'development')
    throw new Error('Synthetic Assurance UI data requires development mode');
  const origin = new URL(process.env.NEXTAUTH_URL ?? '');
  if (origin.protocol !== 'http:' || origin.hostname !== '127.0.0.1')
    throw new Error('Synthetic Assurance UI data requires a loopback HTTP origin');
  if (!['mixed', 'healthy', 'incident', 'empty', 'pending', 'expired'].includes(scenario))
    throw new Error(`Unknown Assurance UI scenario: ${scenario}`);
  return scenario;
}
