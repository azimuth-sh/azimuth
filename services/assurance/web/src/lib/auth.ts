import 'server-only';
import { getServerSession, type NextAuthOptions } from 'next-auth';
import GitHubProvider from 'next-auth/providers/github';
import { redirect, notFound } from 'next/navigation';
import { cache } from 'react';
import { webConfig } from './config';
import { developmentScenario } from './development';

// Project access is re-read from the configured users on every request, so a long session does
// not keep access after a user is removed. Page renders cannot re-issue the cookie, so the
// lifetime counts from sign-in.
const sessionLifetimeSeconds = 7 * 24 * 60 * 60;

export function authOptions(): NextAuthOptions {
  const config = webConfig();
  return {
    secret: process.env.NEXTAUTH_SECRET,
    useSecureCookies: true,
    session: { strategy: 'jwt', maxAge: sessionLifetimeSeconds },
    jwt: { maxAge: sessionLifetimeSeconds },
    providers: [
      GitHubProvider({
        clientId: config.github.clientId,
        clientSecret: config.github.clientSecret,
        checks: ['pkce', 'state'],
        authorization: {
          params: { scope: 'read:user', allow_signup: 'false' },
        },
      }),
    ],
    pages: { signIn: '/login', error: '/login' },
    callbacks: {
      async signIn({ account, user }) {
        return (
          account?.provider === 'github' &&
          Object.hasOwn(webConfig().users, user.id)
        );
      },
      async jwt({ token, account, user }) {
        if (account?.provider === 'github') token.sub = user.id;
        return token;
      },
      async session({ session, token }) {
        const current = webConfig();
        session.githubId = token.sub ?? '';
        session.projects = Object.hasOwn(current.users, session.githubId)
          ? current.users[session.githubId].projects
          : [];
        return session;
      },
      async redirect({ url, baseUrl }) {
        if (url.startsWith('/') && !url.startsWith('//'))
          return `${baseUrl}${url}`;
        if (new URL(url).origin === baseUrl) return url;
        return baseUrl;
      },
    },
  };
}
// Every project read verifies the viewer, and one page makes several reads.
export const viewer = cache(async () => {
  if (developmentScenario()) return {
    githubId: 'synthetic-local-viewer', projects: ['payments-demo'],
    expires: '2099-01-01T00:00:00.000Z',
  };
  const session = await getServerSession(authOptions());
  return session?.githubId && session.projects.length ? session : null;
});
export async function requireViewer(project?: string) {
  const session = await viewer();
  if (!session) redirect('/login');
  if (project && !session.projects.includes(project)) notFound();
  return session;
}
