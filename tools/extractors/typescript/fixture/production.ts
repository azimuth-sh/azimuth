// Synthetic fixture for the emitter's own tests (D2).
import { realizes } from '@azimuth/annotations';

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
