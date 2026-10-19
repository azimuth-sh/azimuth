'use client';

import Link from 'next/link';
import { usePathname, useSearchParams } from 'next/navigation';
export function ProjectNavigation({ root, environment }: { root: string; environment: string }) {
  const pathname = usePathname();
  const selected = useSearchParams().get("environment") ?? environment;
  const links = [
    { href: root, label: 'Overview', icon: '◫' },
    { href: `${root}/model`, label: 'Model', icon: '◇' },
    { href: `${root}/releases`, label: 'Releases', icon: '↗' },
    { href: `${root}/records`, label: 'Records', icon: '☷' },
  ];
  return <nav aria-label="Project navigation">{links.map(link => <Link key={link.href} href={`${link.href}?environment=${encodeURIComponent(selected)}`} aria-current={pathname === link.href || link.href !== root && pathname.startsWith(link.href + '/') ? 'page' : undefined}><span aria-hidden="true">{link.icon}</span>{link.label}</Link>)}</nav>;
}
