"use server";
import { headers } from "next/headers";
import { revalidatePath } from "next/cache";
import { requireViewer } from "@/lib/auth";
import { webConfig } from "@/lib/config";
import { developmentScenario } from "@/lib/development";
import { routeFingerprint } from "@/lib/assurance";

export async function acceptRelease(
  project: string,
  release: string,
  baseline: string,
  _previous: { error: string | null },
  form: FormData,
): Promise<{ error: string | null }> {
  const viewer = await requireViewer(project);
  if (developmentScenario())
    return { error: "Synthetic development data cannot be accepted." };
  const origin = (await headers()).get("origin");
  if (!origin || origin !== process.env.NEXTAUTH_URL?.replace(/\/$/, ""))
    return { error: "Acceptance requires a same-origin request." };
  if (!routeFingerprint(release) || !routeFingerprint(baseline))
    return { error: "Invalid release or baseline identity." };
  const environment = form.get("environment");
  if (
    typeof environment !== "string" ||
    !/^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(environment)
  )
    return { error: "Select a valid deployment environment." };
  const reason = form.get("reason");
  if (typeof reason !== "string" || !reason.trim() || reason.length > 16384)
    return { error: "Enter an acceptance reason of at most 16384 characters." };
  const token = webConfig().users[viewer.githubId]?.ownerTokens?.[project];
  if (!token)
    return { error: "You are not configured as an owner of this project." };
  let response: Response;
  try {
    response = await fetch(
      `${process.env.ASSURANCE_API_URL}/v1/projects/${encodeURIComponent(project)}/releases/${encodeURIComponent(release)}/acceptance`,
      {
        method: "POST",
        headers: {
          authorization: `Bearer ${token}`,
          "content-type": "application/json",
        },
        body: JSON.stringify({
          environment,
          expected_accepted_model_revision: baseline,
          reason: reason.trim(),
        }),
        redirect: "error",
        cache: "no-store",
        signal: AbortSignal.timeout(15000),
      },
    );
  } catch {
    return {
      error:
        "The outcome could not be confirmed. Reload the release before retrying.",
    };
  }
  if (!response.ok)
    return {
      error:
        response.status === 409
          ? "Acceptance preconditions changed or required evidence is missing. Reload and inspect the release."
          : `Acceptance was rejected (HTTP ${response.status}).`,
    };
  revalidatePath(`/projects/${encodeURIComponent(project)}`);
  revalidatePath(
    `/projects/${encodeURIComponent(project)}/releases/${encodeURIComponent(release)}`,
  );
  return { error: null };
}
