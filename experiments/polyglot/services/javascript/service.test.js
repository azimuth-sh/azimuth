const assert = require('node:assert/strict');
const { test } = require('node:test');
const { covers } = require('../../../../packages/typescript/dist/index.js');
const { identity } = require('./service.js');

test('identity is javascript', () => {
  covers('polyglot/identity', 'javascript-identifies', 'unit', 'example', 'direct');
  assert.equal(identity(), 'javascript');
});
