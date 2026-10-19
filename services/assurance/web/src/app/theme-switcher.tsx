'use client';

import { useState } from 'react';
import { themeCookie, themePreference, type Theme } from '@/lib/theme';

export function ThemeSwitcher({ initialTheme }: { initialTheme: Theme }) {
  const [theme, setTheme] = useState(initialTheme);

  function changeTheme(value: string) {
    const preference = themePreference(value);
    document.documentElement.dataset.theme = preference;
    document.cookie = `${themeCookie}=${preference}; Path=/; Max-Age=31536000; SameSite=Lax${location.protocol === 'https:' ? '; Secure' : ''}`;
    setTheme(preference);
  }

  return (
    <label className="themeControl">
      <span>Theme</span>
      <select
        aria-label="Color theme"
        value={theme}
        onChange={(event) => changeTheme(event.target.value)}
      >
        <option value="system">System</option>
        <option value="light">Light</option>
        <option value="dark">Dark</option>
      </select>
    </label>
  );
}
