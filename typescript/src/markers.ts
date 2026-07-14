//! Azimuth linkage markers for TypeScript — the code-side seam that `rtm` audits against the spec.
//!
//! The frontend is functions (Server Components, route handlers), not classes, so the markers are
//! typed no-op *function calls*, not decorators (decorators are class-member-only). They exist to
//! be (a) type-checked at author time and (b) found statically by the emitter, which resolves each
//! call's enclosing named symbol as the `site`. At runtime they do nothing.
//!
//! `exposes`/`upholds` are spec-side, not code tags — they live in the markdown spec, never here.

export type Scope = 'unit' | 'component' | 'e2e';
export type Quantification = 'example' | 'invariant';
export type Oracle = 'direct' | 'golden' | 'metamorphic' | 'model-based' | 'contract';

/**
 * Mark a production-code site as being on a scenario's path. No form: form is how a *test* checks,
 * not a property of code. Keyed by the stable (spec, req, scenario) triple.
 */
export function realizes(spec: string, req: string, scenario: string): void {
  void spec;
  void req;
  void scenario;
}

/**
 * Mark a test as verifying a scenario at a declared form. Form is the pair (scope, quantification);
 * `oracle` is an optional, descriptive label — never gated.
 */
export function covers(
  spec: string,
  req: string,
  scenario: string,
  scope: Scope,
  quantification: Quantification,
  oracle?: Oracle,
): void {
  void spec;
  void req;
  void scenario;
  void scope;
  void quantification;
  void oracle;
}
