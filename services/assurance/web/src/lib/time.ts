const options: Intl.DateTimeFormatOptions = {
  day: 'numeric', month: 'short', year: 'numeric', hour: '2-digit', minute: '2-digit', timeZoneName: 'short',
};
function parse(value: string | number | null | undefined) {
  const date = value === null || value === undefined ? new Date(NaN) : new Date(value);
  return Number.isNaN(date.getTime()) ? null : date;
}
export function formatTime(value: string | number | null | undefined, timeZone?: string) {
  const date = parse(value);
  return date ? new Intl.DateTimeFormat('en-GB', { ...options, timeZone }).format(date) : null;
}
export function isoTime(value: string | number | null | undefined) {
  return parse(value)?.toISOString();
}
