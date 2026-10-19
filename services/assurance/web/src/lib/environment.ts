import "server-only";
import { cookies } from "next/headers";
import { notFound } from "next/navigation";

export async function selectedEnvironment(project: string, value?: string | string[]) {
  if (Array.isArray(value)) notFound();
  const saved = (await cookies()).get(`assurance-environment-${encodeURIComponent(project)}`)?.value;
  let remembered: string | undefined;
  try { remembered = saved ? decodeURIComponent(saved) : undefined; } catch { remembered = undefined; }
  const environment = value ?? remembered ?? "production";
  if (!environment || environment.length > 128 || /[\x00-\x20\x7f]/.test(environment)) notFound();
  return environment;
}
