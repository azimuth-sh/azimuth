import type { Metadata } from 'next';
import Link from 'next/link';
import './globals.css';

export const metadata: Metadata = {
  title: 'Azimuth Assurance',
  description: 'Diagnostic view of qualification, execution, and lifecycle gates.',
};

export default function RootLayout({ children }: Readonly<{ children: React.ReactNode }>) {
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
              <small>Assurance ledger</small>
            </span>
          </Link>
          <p className="boundary">Repository meaning · execution facts · derived gates</p>
        </header>
        <main>{children}</main>
        <footer>
          <span>Reference service</span>
          <span>Decisions explain themselves.</span>
        </footer>
      </body>
    </html>
  );
}
