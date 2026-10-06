import { webConfig } from '@/lib/config';
export const dynamic = 'force-dynamic';
export async function GET() {
  try {
    webConfig();
    return Response.json({ status: 'ok' });
  } catch {
    return Response.json({ status: 'unavailable' }, { status: 503 });
  }
}
