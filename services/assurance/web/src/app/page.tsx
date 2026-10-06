import Link from 'next/link';
import { requireViewer } from '@/lib/auth';
import { SignOut } from './signout';
export const dynamic = 'force-dynamic';
export default async function Home() {
  const session = await requireViewer();
  return (
    <div className="shell">
      <section className="hero">
        <p className="eyebrow">
          Inspection only · GitHub ID {session.githubId}
        </p>
        <h1>Assurance for exact Subjects</h1>
        <p className="lede">
          Inspect current authority, Runs and independent reviews. An accepted
          execution does not establish a supported Claim.
        </p>
        <SignOut />
      </section>
      <section className="section">
        <h2>Your projects</h2>
        <div className="projectGrid">
          {session.projects.map((id) => (
            <Link
              className="projectCard"
              href={`/projects/${encodeURIComponent(id)}`}
              key={id}
            >
              <h3>{id}</h3>
              <span className="projectAction">Inspect account →</span>
            </Link>
          ))}
        </div>
      </section>
    </div>
  );
}
