// Synthetic fixture for the emitter's own tests.
import { realizes } from '@azimuth-sh/annotations';

export function handler(): string {
  realizes('alpha', 'route-thing');
  return 'ok';
}

export const projection = (phase: string): string => {
  realizes('alpha', 'projection-thing');
  realizes('alpha', 'second-claim');
  return phase;
};

export function untagged(): void {}
