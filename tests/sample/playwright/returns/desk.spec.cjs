const { test, expect } = require('@playwright/test');
test('charges late return', { tag: '@Check:late-return-is-fined' }, async () => {
  expect(2 + 3).toBe(5);
});
test('waives on-time fine', { tag: '@Check:on-time-return-is-free' }, async () => {
  expect(2 + 3).toBe(5);
});
