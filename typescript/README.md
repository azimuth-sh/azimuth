# `@azimuth/annotations`

Azimuth linkage markers for TypeScript, plus a static-scan **manifest emitter** for
[`rtm`](../rtm). This is the polyglot path: your codebase emits a language-neutral
`*.manifest.json` (see [`schema/manifest.schema.json`](../schema/manifest.schema.json)) and `rtm`
ingests it the same as any other language's.

## Why marker calls, not decorators

The frontend is **functions** — Server Components, route handlers — not classes. TypeScript
decorators are class-member-only, so they can't tag a `function GET()` or a `const Page = …`.
Instead the markers are typed **no-op function calls** you place in the body of the code or test
they describe:

- `realizes(spec, req, scenario)` — on **production code**, "this site is on that scenario's path."
  No form: form is how a *test* checks, not a property of code.
- `covers(spec, req, scenario, scope, quantification, oracle?)` — on a **test**, "this verifies
  that scenario, at this form."

They do nothing at runtime. They exist to (a) type-check at author time — `scope`,
`quantification`, and `oracle` are string-literal unions, so a typo is a compile error — and (b) be
found by the emitter, which resolves each call's **enclosing named symbol** as the `site`.

> `exposes` / `upholds` are **spec-side**, declared in the markdown spec — they are not code tags
> and have no marker here.

```ts
import { realizes, covers } from '@azimuth/annotations';

export async function GET(): Promise<Response> {
  realizes('public-certificates', 'detail', 'detail-valid');
  return Response.json(await loadCertificate());
}

test('revoked certificate returns 404', () => {
  covers('public-certificates', 'detail', 'detail-revoked-void', 'component', 'invariant', 'direct');
  // …assertions…
});
```

### Form axes

`scope ∈ unit | component | e2e` — how much of the real system runs.
`quantification ∈ example | invariant` — one case (∃) vs a property over all inputs/states (∀).
`oracle ∈ direct | golden | metamorphic | model-based | contract` — optional, **descriptive only,
never gated** (how the expected result was obtained). `contract` is an oracle for a cross-service
seam, not a scope.

## Install (consumer)

Not yet published to a registry — reference it locally.

```jsonc
// package.json
{
  "dependencies": {
    "@azimuth/annotations": "file:../azimuth/typescript"
  }
}
```

```sh
npm install
# or, without editing package.json:
npm install /path/to/azimuth/typescript
# or link it for local dev:
npm link /path/to/azimuth/typescript
```

Then `import { realizes, covers } from '@azimuth/annotations';`.

## Emit a manifest

The emitter scans `.ts`/`.tsx` sources for marker calls and writes a manifest.

```sh
# via the installed bin (resolves under node_modules/.bin):
azimuth-emit --root . --out artifacts/frontend.manifest.json 'src/**/*.ts' 'src/**/*.tsx'

# or straight from this package during development:
npm run emit -- --root ../some-frontend --out /tmp/frontend.manifest.json 'src/**/*.tsx'
```

| Flag | Default | Meaning |
|---|---|---|
| `--root <dir>` | cwd | Codebase root; `file` paths in the manifest are relative to it. |
| `--out <file>` / `-o` | stdout | Where to write the manifest JSON (parent dirs created). |
| `<glob>…` | `**/*.ts`, `**/*.tsx` | Include globs, resolved under `--root`. `.d.ts` files are skipped. |

The manifest is deterministic (entries sorted by file, site, then the id triple). Marker calls the
scanner can't resolve — a non-string-literal argument, too few arguments, or an unknown
`scope`/`quantification`/`oracle` — are reported to **stderr** and skipped; they do not fail the run.

### Site resolution

The `site` is the **nearest enclosing named symbol** above the marker call:

- a named `function`/method declaration → its name (e.g. a route handler `GET`);
- a `const`/`let` binding of an arrow or function → the binding name (e.g. a Server Component
  `const CertificatePage = …`);
- a `test(…)` / `it(…)` / `describe(…)` call whose first argument is a string → that description
  (how TypeScript tests name themselves, since they are calls, not named functions).

Nearest wins, so `covers` inside `test('…', () => …)` names the test while `realizes` inside
`export function GET` names the handler. If nothing named encloses the call, the site is `unknown`.

## Develop

```sh
npm install
npm run build   # tsc → dist/ (with the azimuth-emit bin)
npm test        # node:test via tsx, scans the sample fixture and asserts the manifest
```

TypeScript strict mode is on. The emitter uses the TypeScript compiler API directly (no extra
parser dependency).
