"use client";
import { useActionState } from "react";
import { acceptRelease } from "@/app/projects/[projectId]/releases/[release]/actions";

export function ReleaseAcceptance({
  project,
  release,
  baseline,
  environments,
}: {
  project: string;
  release: string;
  baseline: string;
  environments: string[];
}) {
  const [state, action, pending] = useActionState(
    acceptRelease.bind(null, project, release, baseline),
    { error: null },
  );
  return (
    <section className="section">
      <h2>Accept composed release</h2>
      <p>
        This owner decision advances the accepted model. It does not deploy,
        apply repository model changes or archive Changes.
      </p>
      <form action={action}>
        <label htmlFor="acceptance-environment">
          Assessed deployment environment
        </label>
        <select
          id="acceptance-environment"
          name="environment"
          required
          defaultValue={
            environments.includes("production") ? "production" : environments[0]
          }
        >
          {environments.map((e) => (
            <option key={e}>{e}</option>
          ))}
        </select>
        <label htmlFor="acceptance-reason">Acceptance reason</label>
        <textarea
          id="acceptance-reason"
          name="reason"
          required
          maxLength={16384}
          rows={3}
        />
        <p>
          <button type="submit" disabled={pending || !environments.length}>
            {pending ? "Accepting…" : "Accept release"}
          </button>
        </p>
        {state.error && <p role="alert">{state.error}</p>}
      </form>
    </section>
  );
}
