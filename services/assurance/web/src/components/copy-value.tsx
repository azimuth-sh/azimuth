'use client';

import { useState } from 'react';
export function CopyValue({ label, value }: { label: string; value: string }) {
  const [message, setMessage] = useState('');
  async function copy() {
    try { await navigator.clipboard.writeText(value); setMessage('Copied'); }
    catch { setMessage('Select and copy the value manually'); }
  }
  return <div className="technicalValue"><span>{label}</span><code>{value}</code><button type="button" onClick={copy} aria-label={`Copy ${label}`}>Copy</button><small role="status">{message}</small></div>;
}
