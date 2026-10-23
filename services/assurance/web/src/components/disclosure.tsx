import type { ReactNode } from 'react';

export function Disclosure({ summary, meta, open, className, children }: {
  summary: ReactNode;
  meta?: ReactNode;
  open?: boolean;
  className?: string;
  children: ReactNode;
}) {
  return <details className={`disclosure${className ? ` ${className}` : ''}`} open={open}>
    <summary><span className="disclosureTitle">{summary}</span>{meta !== undefined && <span className="disclosureMeta">{meta}</span>}</summary>
    <div className="disclosureBody">{children}</div>
  </details>;
}
