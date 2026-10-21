"use client";

import { usePathname, useRouter, useSearchParams } from "next/navigation";
import { useEffect, useTransition } from "react";

export function EnvironmentSelector({ project, initial, environments }: {
  project: string; initial: string; environments: string[];
}) {
  const query = useSearchParams();
  const pathname = usePathname();
  const router = useRouter();
  const [pending, startTransition] = useTransition();
  const selected = query.get("environment") ?? initial;
  const options = [...new Set([...environments, selected])];
  useEffect(() => {
    document.cookie = `assurance-environment-${encodeURIComponent(project)}=${encodeURIComponent(selected)}; Path=/; Max-Age=31536000; SameSite=Lax${location.protocol === 'https:' ? '; Secure' : ''}`;
  }, [project, selected]);
  return <div className="environmentSelector"><label htmlFor="workspace-environment">Environment</label><select id="workspace-environment" value={selected} disabled={pending} onChange={event => {
    const environment = event.target.value;
    document.cookie = `assurance-environment-${encodeURIComponent(project)}=${encodeURIComponent(environment)}; Path=/; Max-Age=31536000; SameSite=Lax${location.protocol === 'https:' ? '; Secure' : ''}`;
    const values = new URLSearchParams(query.toString());
    values.set("environment", environment);
    values.delete("deployment");
    values.delete("verification-target");
    if (values.get("context")?.startsWith("deployment:")) {
      const release = values.get("release");
      if (release) values.set("context", `release:${release}`);
      else values.delete("context");
    }
    startTransition(() => { router.push(`${pathname}?${values}`); router.refresh(); });
  }}>{options.map(environment => <option value={environment} key={environment}>{environment}</option>)}</select></div>;
}
