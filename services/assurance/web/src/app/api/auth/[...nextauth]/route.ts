import NextAuth from 'next-auth';
import type { NextRequest } from 'next/server';
import { authOptions } from '@/lib/auth';
export const dynamic = 'force-dynamic';
export const runtime = 'nodejs';
async function handler(
  request: NextRequest,
  context: { params: Promise<{ nextauth: string[] }> },
) {
  return NextAuth(authOptions())(request, context);
}
export { handler as GET, handler as POST };
