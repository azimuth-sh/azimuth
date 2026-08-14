import { covers } from '@azimuth/annotations';

declare function test(name: string, body: () => void): void;

test('the route answers', () => {
  covers('alpha', 'route-thing', 'component', 'universal');
});

test('the projection redacts', () => {
  covers('alpha', 'projection-thing', 'e2e', 'example', 'model-based');
});

test('the harness boots', () => {
  const ready = true;
  void ready;
});

test('a bare test declaring nothing', () => {
  const x = 1;
  void x;
});
