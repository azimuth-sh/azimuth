const assert = require('node:assert/strict');
const { test } = require('node:test');
const { identity } = require('./service.js');

test('identity is javascript', () => {
  assert.equal(identity(), 'javascript');
});
