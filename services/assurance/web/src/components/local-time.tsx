'use client';

import { useEffect, useState } from 'react';
import { formatTime, isoTime } from '../lib/time';

// The server renders in UTC; the viewer's own timezone is only known after hydration.
export function LocalTime({ value, fallback = 'Time unavailable', className }: {
  value: string | number | null | undefined;
  fallback?: string;
  className?: string;
}) {
  const [local, setLocal] = useState<string | null>(null);
  useEffect(() => setLocal(formatTime(value)), [value]);
  const text = local ?? formatTime(value, 'UTC') ?? fallback;
  return <time className={className} dateTime={isoTime(value)} title={isoTime(value)}>{text}</time>;
}
