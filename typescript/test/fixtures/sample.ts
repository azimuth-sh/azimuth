import { covers, realizes } from '../../src/markers';

// A route handler (production code) on a scenario's path — site resolves to the export name.
export async function GET(): Promise<Response> {
  realizes('public-certificates', 'detail', 'detail-valid');
  return new Response('ok');
}

// A const-bound Server Component — site resolves to the binding name.
export const CertificatePage = async (): Promise<string> => {
  realizes('public-certificates', 'detail', 'detail-valid');
  return 'page';
};

// A test, named by its description string.
test('revoked certificate returns 404', () => {
  covers('public-certificates', 'detail', 'detail-revoked-void', 'component', 'invariant', 'direct');
});

// A test without an oracle — oracle is optional.
it('unpublished is an indistinguishable 404', () => {
  covers('public-certificates', 'detail', 'detail-unpublished', 'component', 'example');
});
