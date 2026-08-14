import Link from 'next/link';

export default function NotFound() {
  return (
    <div className="shell narrow">
      <section className="hero">
        <p className="eyebrow">404</p>
        <h1>Assurance account not found</h1>
        <p className="lede">The project may not have been registered in this ledger.</p>
        <Link href="/" className="buttonLink">
          Return to projects
        </Link>
      </section>
    </div>
  );
}
