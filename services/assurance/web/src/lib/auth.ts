import 'server-only';
import { getServerSession, type NextAuthOptions } from 'next-auth';
import GitHubProvider from 'next-auth/providers/github';
import { redirect, notFound } from 'next/navigation';
import { webConfig } from './config';

export function authOptions(): NextAuthOptions {
  const config = webConfig();
  return {
    secret: process.env.NEXTAUTH_SECRET,
    useSecureCookies: true,
    session: { strategy: 'jwt', maxAge: 3600 },
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
export async function viewer() {
  const session = await getServerSession(authOptions());
  return session?.githubId && session.projects.length ? session : null;
}
export async function requireViewer(project?: string) {
  const session = await viewer();
  if (!session) redirect('/login');
  if (project && !session.projects.includes(project)) notFound();
  return session;
}
