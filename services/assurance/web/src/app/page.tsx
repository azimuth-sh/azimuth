import Link from 'next/link';
import { getProjects, Project } from '@/lib/assurance';

export default async function Home() {
  let projects: Project[];
  let failure: string | null = null;
  try {
    projects = await getProjects();
  } catch (error) {
    projects = [];
    failure = error instanceof Error ? error.message : 'The assurance API is unavailable.';
  }

  return (
    <div className="shell">
      <section className="hero">
        <p className="eyebrow">Continuous assurance, without continuous Git churn</p>
        <h1>What is justified for this exact thing, right now?</h1>
        <p className="lede">
          Qualified definitions stay stable. CI and production observations renew execution state.
          Every closed gate names the missing fact or decision.
        </p>
      </section>

      <section className="section" aria-labelledby="projects-heading">
        <div className="sectionHeading">
          <div>
            <p className="kicker">Projects</p>
            <h2 id="projects-heading">Assurance accounts</h2>
          </div>
          <span className="count">{projects.length}</span>
        </div>

        {failure ? (
          <div className="notice noticeError">
            <strong>Service unavailable</strong>
            <p>{failure}</p>
          </div>
        ) : projects.length === 0 ? (
          <div className="empty">
            <p>No projects have been registered.</p>
            <code>POST /v1/projects</code>
          </div>
        ) : (
          <div className="projectGrid">
            {projects.map((project) => (
              <Link className="projectCard" href={`/projects/${encodeURIComponent(project.id)}`} key={project.id}>
                <span className="projectId">{project.id}</span>
                <h3>{project.name}</h3>
                <span className="projectAction">Inspect account →</span>
              </Link>
            ))}
          </div>
        )}
      </section>
    </div>
  );
}
