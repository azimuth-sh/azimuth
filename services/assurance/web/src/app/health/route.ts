import { webConfig } from '@/lib/config';
import { developmentScenario } from '@/lib/development';
export const dynamic = 'force-dynamic';
export async function GET() {
  try {
    if (developmentScenario()) return Response.json({ status: 'ok', data: 'synthetic' });
    webConfig();
    return Response.json({ status: 'ok' });
  } catch {
    return Response.json({ status: 'unavailable' }, { status: 503 });
  }
}
