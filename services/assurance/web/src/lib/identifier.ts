export function shortIdentifier(value: string, length = 8) {
  const bare = value.replace(/^sha256:/, '');
  if (/^[0-9a-f]{40,}$/.test(bare)) return bare.slice(0, length);
  const prefixed = /^([a-z][a-z-]*-)([0-9a-f]{40,})$/.exec(value);
  if (prefixed) return `${prefixed[1]}${prefixed[2].slice(0, length)}`;
  return value.length > 28 ? `${value.slice(0, 24)}…` : value;
}
