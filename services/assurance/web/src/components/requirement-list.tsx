'use client';

import Link from 'next/link';
import { useState, type MouseEvent } from 'react';

export type RequirementStatus = 'supported' | 'violated' | 'unresolved' | 'not-applicable';

export interface RequirementRow {
  id: string;
  title: string;
  summary: string;
  status: RequirementStatus;
  module: string;
  details: string;
}

const filters = [
  { id: 'all', label: 'All Requirements' },
  { id: 'supported', label: 'Satisfied' },
  { id: 'violated', label: 'Not satisfied' },
  { id: 'unresolved', label: 'Unresolved' },
  { id: 'not-applicable', label: 'Not applicable here' },
] as const;
const statusLabels = { supported: 'Satisfied', violated: 'Not satisfied', unresolved: 'Unresolved', 'not-applicable': 'Not applicable here' };
const tones = { supported: 'good', violated: 'bad', unresolved: 'warn', 'not-applicable': 'neutral' };
const marks = { supported: '✓', violated: '!', unresolved: '?', 'not-applicable': '–' };

/** Every Requirement of the deployment is already on the page, so switching the status filter only
 * narrows rows locally; the address is kept in step so the filtered view stays linkable. */
export function RequirementList({ rows, environment, initialStatus, filterHrefs }: {
  rows: RequirementRow[];
  environment: string;
  initialStatus: string;
  filterHrefs: Record<string, string>;
}) {
  const [status, setStatus] = useState(initialStatus);
  const filter = filters.find(item => item.id === status) ?? filters[0];
  const matching = rows.filter(row => status === 'all' || row.status === status);
  const groups = new Map<string, RequirementRow[]>();
  for (const row of matching) groups.set(row.module, [...(groups.get(row.module) ?? []), row]);
  function select(event: MouseEvent<HTMLAnchorElement>, id: string) {
    if (event.defaultPrevented || event.button !== 0 || event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) return;
    event.preventDefault();
    setStatus(id);
    window.history.replaceState(null, '', filterHrefs[id]);
  }
  return <>
    <header className="dashboardHeading"><div><p className="overline">ENVIRONMENT REQUIREMENTS</p><h1>{filter.id === 'all' ? 'All Requirements' : `${filter.label} Requirements`}</h1><p>Grouped by module · {matching.length} matching {matching.length === 1 ? 'Requirement' : 'Requirements'}</p></div><span className="environmentBadge">{environment}</span></header>
    <nav className="requirementFilters" aria-label="Filter Requirements by Requirement Status">{filters.map(item => <a key={item.id} href={filterHrefs[item.id]} onClick={event => select(event, item.id)} aria-current={item.id === filter.id ? 'page' : undefined}>{item.label}<span>{item.id === 'all' ? rows.length : rows.filter(row => row.status === item.id).length}</span></a>)}</nav>
    {!matching.length ? <section className="dashboardPanel panelEmpty"><h2>No {filter.id === 'all' ? '' : `${filter.label.toLowerCase()} `}Requirements</h2><p>{rows.length ? 'No Requirements match this status in the selected deployment. Choose another status above.' : 'No Requirements have been assessed for this deployment.'}</p></section> : <div className="requirementGroups">{[...groups].map(([module, group]) => <section className="dashboardPanel requirementGroup" key={module}><div className="panelHeading"><h2><span className="moduleIcon" aria-hidden="true">◇</span>{module}</h2><span className="panelCount">{group.length}</span></div>{group.map(row => <Link className="attentionRow" key={row.id} href={row.details}><span className={`attentionIcon ${tones[row.status]}`} aria-hidden="true">{marks[row.status]}</span><div><div className="rowMeta"><span className={`badge ${tones[row.status]}`}>{statusLabels[row.status]}</span></div><h3>{row.title}</h3><p>{row.summary}</p><span className="rowAction">View Requirement, review & evidence ↗</span></div></Link>)}</section>)}</div>}
  </>;
}
