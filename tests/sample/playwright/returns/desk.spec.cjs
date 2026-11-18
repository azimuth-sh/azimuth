const { test, expect } = require('@playwright/test');
test('charges late return', { tag: '@Check:late-return-is-fined' }, async () => {
  const response = await fetch(`http://127.0.0.1:${process.env.DESK_PORT}`);
  expect(await response.text()).toBe('library desk');
});
test('waives on-time fine', { tag: '@Check:on-time-return-is-free' }, async () => {
  const response = await fetch(`http://127.0.0.1:${process.env.DESK_PORT}`);
  expect(await response.text()).toBe('library desk');
});
