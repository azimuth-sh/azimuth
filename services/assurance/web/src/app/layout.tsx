import type { Metadata } from 'next';
import { cookies } from 'next/headers';
import { themeCookie, themePreference } from '@/lib/theme';
import { ThemeSwitcher } from './theme-switcher';
import Link from 'next/link';
import './globals.css';
import { developmentScenario } from '@/lib/development';

export const metadata: Metadata = {
  title: 'Azimuth Assurance',
  description:
    'Private inspection of exact-Subject Assurance State, independent reviews and Runs.',
};

export default async function RootLayout({
  children,
}: Readonly<{ children: React.ReactNode }>) {
  const theme = themePreference((await cookies()).get(themeCookie)?.value);
  return (
    <html lang="en" data-theme={theme} data-scroll-behavior="smooth">
      <body>
        <header className="masthead">
          <Link href="/" className="brand" aria-label="Azimuth Assurance home">
            <span className="brandMark" aria-hidden="true">
              A
            </span>
            <span>
              <strong>Azimuth</strong>
              <small>Assurance State</small>
            </span>
          </Link>
          <div className="headerActions">
            <p className="boundary">
              Explicit authority · execution facts · independent reviews
            </p>
            <ThemeSwitcher initialTheme={theme} />
          </div>
        </header>
        {developmentScenario() && <aside role="status" className="developmentBanner">
          <strong>Synthetic local data · {developmentScenario()}</strong>
          <span>UI development only. No production connection or validated assurance records.</span>
        </aside>}
        <main>{children}</main>
        <footer>
          <span>Inspection only</span>
          <span>Decisions explain themselves.</span>
        </footer>
      </body>
    </html>
  );
}
