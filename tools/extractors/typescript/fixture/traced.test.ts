import { implementsCheck } from '@azimuth-sh/annotations';

declare function test(name: string, body: () => void): void;

test('the route answers', () => {
  implementsCheck('alpha/route-answer');
});

test('the projection redacts', () => {
  implementsCheck('alpha/projection-redaction');
});

test('the harness boots', () => {
  const ready = true;
  void ready;
});

test('a bare test declaring nothing', () => {
  const x = 1;
  void x;
});
