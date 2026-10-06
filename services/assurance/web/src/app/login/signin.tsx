'use client';
import { signIn } from 'next-auth/react';
export function SignIn() {
  return (
    <button
      type="button"
      onClick={() => void signIn('github', { callbackUrl: '/' })}
    >
      Sign in with GitHub
    </button>
  );
}
