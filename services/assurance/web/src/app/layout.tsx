import type { Metadata } from 'next';
import Link from 'next/link';
import './globals.css';

export const metadata: Metadata = {
  title: 'Azimuth Assurance',
  description:
    'Private inspection of exact-Subject Assurance State, independent reviews and Runs.',
};

export default function RootLayout({
  children,
}: Readonly<{ children: React.ReactNode }>) {
  return (
    <html lang="en">
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
          <p className="boundary">
            Explicit authority · execution facts · independent reviews
          </p>
        </header>
        <main>{children}</main>
        <footer>
          <span>Inspection only</span>
          <span>Decisions explain themselves.</span>
        </footer>
      </body>
    </html>
  );
}
