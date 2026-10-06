import type { NextRequest } from 'next/server';
import { viewer } from '@/lib/auth';
import { readProject } from '@/lib/assurance';
export const dynamic = 'force-dynamic';
export async function GET(
  request: NextRequest,
  { params }: { params: Promise<{ project: string; resource: string[] }> },
) {
  const { project, resource } = await params;
  const session = await viewer();
  if (!session)
    return Response.json({ error: 'Authentication required' }, { status: 401 });
  if (!session.projects.includes(project))
    return Response.json({ error: 'Not found' }, { status: 404 });
  const valid =
    (resource.length === 1 &&
      ['authority', 'runs', 'subjects', 'reviews'].includes(resource[0])) ||
    (resource.length === 2 && resource[0] === 'runs') ||
    (resource.length === 3 &&
      resource[0] === 'subjects' &&
      resource[2] === 'state' &&
      /^sha256:[a-f0-9]{64}$/.test(resource[1]));
  if (!valid) return Response.json({ error: 'Not found' }, { status: 404 });
  const query = request.nextUrl.searchParams;
  if ([...query.keys()].some((key) => !['after', 'limit'].includes(key)))
    return Response.json({ error: 'Invalid query' }, { status: 400 });
  const result = await readProject(
    project,
    `/${resource.map(encodeURIComponent).join('/')}${query.size ? '?' + query.toString() : ''}`,
  );
  return Response.json(result ?? { error: 'Not found' }, {
    status: result ? 200 : 404,
    headers: { 'cache-control': 'no-store' },
  });
}
