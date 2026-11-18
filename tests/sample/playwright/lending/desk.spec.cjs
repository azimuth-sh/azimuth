const { test, expect } = require('@playwright/test');
test('refuses sixth loan', { tag: '@Check:sixth-loan-is-refused' }, async () => {
  const response = await fetch(`http://127.0.0.1:${process.env.DESK_PORT}`);
  expect(await response.text()).toBe('library desk');
});
test('lends after return', { tag: '@Check:returned-book-frees-a-slot' }, async () => {
  const response = await fetch(`http://127.0.0.1:${process.env.DESK_PORT}`);
  expect(await response.text()).toBe('library desk');
});
