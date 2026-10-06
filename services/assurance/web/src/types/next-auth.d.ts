import 'next-auth';
declare module 'next-auth' {
  interface Session {
    githubId: string;
    projects: string[];
  }
}
