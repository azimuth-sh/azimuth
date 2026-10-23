'use client';

import { useState } from 'react';
import { shortIdentifier } from '../lib/identifier';

export function Identifier({ value, label }: { value: string; label?: string }) {
  const [copied, setCopied] = useState(false);
  async function copy() {
    try { await navigator.clipboard.writeText(value); setCopied(true); setTimeout(() => setCopied(false), 1500); }
    catch { setCopied(false); }
  }
  return <span className="identifier">
    <code title={value}>{label ?? shortIdentifier(value)}</code>
    <button type="button" onClick={copy} aria-label={`Copy ${label ?? 'identifier'} ${value}`}>{copied ? 'Copied' : 'Copy'}</button>
  </span>;
}
