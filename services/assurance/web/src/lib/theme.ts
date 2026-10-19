export type Theme = 'light' | 'dark' | 'system';

export const themeCookie = 'assurance-theme';

export function themePreference(value: string | undefined): Theme {
  return value === 'light' || value === 'dark' ? value : 'system';
}
