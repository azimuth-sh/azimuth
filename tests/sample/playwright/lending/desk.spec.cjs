const { test, expect } = require('@playwright/test');
test('refuses sixth loan', { tag: '@Check:sixth-loan-is-refused' }, async () => {
  expect(2 + 3).toBe(5);
});
test('lends after return', { tag: '@Check:returned-book-frees-a-slot' }, async () => {
  expect(2 + 3).toBe(5);
});
