// No ImplementsCheck marker, so this ordinary test stays outside Azimuth.
declare function test(name: string, body: () => void): void;

test('a bare test in a non-tracing file', () => {});
