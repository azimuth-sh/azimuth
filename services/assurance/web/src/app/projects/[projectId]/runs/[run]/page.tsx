import Link from 'next/link';
import { notFound } from 'next/navigation';
import { requireViewer } from '@/lib/auth';
import { cursor, readProject } from '@/lib/assurance';
export const dynamic = 'force-dynamic';
export default async function RunPage({
  params,
  searchParams,
}: {
  params: Promise<{ projectId: string; run: string }>;
  searchParams: Promise<Record<string, string | string[] | undefined>>;
}) {
  const { projectId, run } = await params;
  await requireViewer(projectId);
  const after = cursor((await searchParams).after);
  const history = await readProject<Record<string, unknown>>(
    projectId,
    `/runs/${encodeURIComponent(run)}?limit=1&after=${encodeURIComponent(after)}`,
  );
  if (!history) notFound();
  return (
    <div className="shell">
      <Link href={`/projects/${encodeURIComponent(projectId)}`}>← Project</Link>
      <h1>Run revision</h1>
      <p>
        Execution facts do not approve reviews or establish a Claim conclusion.
      </p>
      <pre>{JSON.stringify(history, null, 2)}</pre>
      {typeof history.next === 'string' && (
        <Link href={`?after=${encodeURIComponent(history.next)}`}>
          Next revision →
        </Link>
      )}
    </div>
  );
}
