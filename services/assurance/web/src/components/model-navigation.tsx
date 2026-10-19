'use client';

import Link from 'next/link';
import { useEffect, useState } from 'react';

type Module = { id: string; title: string; parent?: string; claims: string[] };
export function ModelNavigation({ modules, selectedModule, base }: {
  modules: Module[];
  selectedModule?: string; base: string;
}) {
  const ancestors: string[] = [];
  let current = modules.find(m => m.id === selectedModule);
  while (current && !ancestors.includes(current.id)) {
    ancestors.push(current.id);
    current = modules.find(m => m.id === current?.parent);
  }
  const ancestorKey = JSON.stringify(ancestors);
  const [expanded, setExpanded] = useState<string[]>(ancestors);
  useEffect(() => {
    const selected: string[] = JSON.parse(ancestorKey);
    setExpanded(values => [...new Set([...values, ...selected])]);
  }, [ancestorKey]);
  function toggle(id: string) {
    setExpanded(values => values.includes(id) ? values.filter(value => value !== id) : [...values, id]);
  }
  function total(id: string, trail: string[] = []): number {
    if (trail.includes(id)) return 0;
    const module = modules.find(m => m.id === id);
    return (module?.claims.length ?? 0) + modules.filter(m => m.parent === id).reduce((sum,m) => sum + total(m.id,[...trail,id]),0);
  }
  const rendered = new Set<string>();
  function branch(module: Module) {
    if (rendered.has(module.id)) return null;
    rendered.add(module.id);
    const children = modules.filter(m => m.parent === module.id && m.id !== module.id);
    const open = expanded.includes(module.id);
    const count = total(module.id);
    return <li key={module.id}>
      <div className={`modelTreeRow${selectedModule === module.id ? ' selectedModule' : ''}`}>
        {children.length ? <button type="button" className="treeToggle" aria-label={`${open ? 'Collapse' : 'Expand'} ${module.title}`} aria-expanded={open} onClick={() => toggle(module.id)}><svg viewBox="0 0 16 16" width="12" height="12" fill="none" aria-hidden="true"><path d="m6 4 4 4-4 4" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" /></svg></button> : <span className="treeToggleSpace" />}
        <Link href={`${base}&module=${encodeURIComponent(module.id)}`} scroll={false} aria-current={selectedModule === module.id ? 'location' : undefined}>
          <svg className="treeModuleIcon" viewBox="0 0 20 20" width="15" height="15" fill="none" aria-hidden="true"><path d="M3 5.5h5l1.5 2H17v8.5H3V5.5Z M3 7.5h14" stroke="currentColor" strokeWidth="1.3" strokeLinejoin="round" /></svg>
          <span className="treeModuleTitle" title={module.title || module.id}>{module.title || module.id}</span>
          <small title={`${count} Claims, including nested modules`} aria-label={`${count} Claims`}>{count}</small>
        </Link>
      </div>
      {open && children.length > 0 && <ul>{children.map(branch)}</ul>}
    </li>;

  }
  const roots = modules.filter(m => !m.parent || !modules.some(parent => parent.id === m.parent));
  const reachable = new Set<string>();
  function mark(id: string) {
    if (reachable.has(id)) return;
    reachable.add(id);
    modules.filter(m => m.parent === id).forEach(m => mark(m.id));
  }
  roots.forEach(m => mark(m.id));
  for (const module of modules) if (!reachable.has(module.id)) { roots.push(module); mark(module.id); }
  return <nav className="modelNavigation" aria-label="Model modules"><p className="fieldLabel">MODULES</p><Link className="modelTreeHome" href={base} aria-current={!selectedModule ? 'page' : undefined}><svg viewBox="0 0 20 20" width="15" height="15" fill="none" aria-hidden="true"><path d="M3 3h5v5H3z M12 3h5v5h-5z M3 12h5v5H3z M12 12h5v5h-5z" stroke="currentColor" strokeWidth="1.2" /></svg><strong>All modules</strong><span>{modules.length}</span></Link><ul className="modelTree">{roots.map(branch)}</ul>{!modules.length && <p className="treeEmpty">No modules in this model.</p>}</nav>;
}
