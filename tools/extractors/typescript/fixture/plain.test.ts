// No covers anywhere, so this file is not tracing and its bare test is not a finding.
declare function test(name: string, body: () => void): void;

test('a bare test in a non-tracing file', () => {});
