import { strict as assert } from 'node:assert';
import { test } from 'node:test';
import * as fs from 'node:fs';
import * as os from 'node:os';
import * as path from 'node:path';
import { surfaceTargets } from './workspace';

test('derives Next surface targets from area mounts', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'azimuth-workspace-'));
  const workspace = path.join(root, 'workspace.json');
  fs.writeFileSync(workspace, JSON.stringify({
    format: 'azimuth-workspace',
    version: 1,
    areas: [{ id: 'rider-experience', mounts: [{ id: 'code', path: 'app/web/rider' }] }],
    surfaces: [{
      id: 'trips/rider-view',
      contributions: [{
        area: 'rider-experience', mount: 'code', enumerator: 'next-routes',
      }],
    }],
  }));

  assert.deepEqual(surfaceTargets(workspace, root), [{
    surface: 'trips/rider-view',
    area: 'rider-experience',
    mount: 'code',
    enumerator: 'next-routes',
    root: path.join(root, 'app/web/rider'),
  }]);
});

test('fails rather than dropping a contribution with an unknown mount', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'azimuth-workspace-'));
  const workspace = path.join(root, 'workspace.json');
  fs.writeFileSync(workspace, JSON.stringify({
    format: 'azimuth-workspace', version: 1, areas: [],
    surfaces: [{ id: 'trips/rider-view', contributions: [{
      area: 'missing', mount: 'code', enumerator: 'next-routes',
    }] }],
  }));

  assert.throws(() => surfaceTargets(workspace, root), /unknown mount missing:code/);
});

test('rejects mount paths that could escape the repository before extraction', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'azimuth-workspace-'));
  const workspace = path.join(root, 'workspace.json');
  fs.writeFileSync(workspace, JSON.stringify({
    format: 'azimuth-workspace', version: 1,
    areas: [{ id: 'web', mounts: [{ id: 'code', path: '../outside' }] }],
    surfaces: [],
  }));

  assert.throws(() => surfaceTargets(workspace, root), /not a normalized relative path/);
});

test('rejects duplicate area mount identities instead of overwriting one', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'azimuth-workspace-'));
  const workspace = path.join(root, 'workspace.json');
  fs.writeFileSync(workspace, JSON.stringify({
    format: 'azimuth-workspace', version: 1,
    areas: [{ id: 'web', mounts: [
      { id: 'code', path: 'app/web/first' },
      { id: 'code', path: 'app/web/second' },
    ] }],
    surfaces: [],
  }));

  assert.throws(() => surfaceTargets(workspace, root), /duplicate mount web:code/);
});
