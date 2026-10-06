import Link from 'next/link';
import { SignIn } from './signin';
export const dynamic = 'force-dynamic';
export default function Login() {
  return (
    <div className="shell">
      <section className="hero">
        <p className="eyebrow">Private inspection</p>
        <h1>Assurance State Service</h1>
        <p className="lede">
          Sign in with an authorized GitHub account. Project access is
          explicitly assigned.
        </p>
        <SignIn />
        <p>
          Access denied? Your stable GitHub user ID must be enrolled by the
          operator.
        </p>
        <Link href="/">Return to projects</Link>
      </section>
    </div>
  );
}
