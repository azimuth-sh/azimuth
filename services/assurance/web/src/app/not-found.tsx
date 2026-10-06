import Link from 'next/link';

export default function NotFound() {
  return (
    <div className="shell narrow">
      <section className="hero">
        <p className="eyebrow">404</p>
        <h1>Record not found</h1>
        <p className="lede">
          The requested record is unavailable or you do not have access to it.
        </p>
        <Link href="/" className="buttonLink">
          Return to projects
        </Link>
      </section>
    </div>
  );
}
