/**
 * Azimuth linkage tags for TypeScript.
 *
 * The front end is functions — route handlers, server components, hooks — not classes, so the tags
 * are typed no-op *function calls* rather than decorators, which are class-member-only. They exist
 * to be type-checked at author time and found statically by the emitter, which resolves each call's
 * enclosing named symbol as the site. At runtime they do nothing.
 */

export type Scope = 'unit' | 'component' | 'e2e';
export type Quantification = 'example' | 'universal';
export type Oracle =
  | 'direct'
  | 'golden'
  | 'relational'
  | 'metamorphic'
  | 'model-based'
  | 'contract';

/**
 * Marks a production-code site as being on a claim's path, keyed by the stable
 * `(spec, scenario)` pair.
 *
 * The pair, not a triple: scenario ids are unique per spec, so a requirement id would be redundant
 * information that can go stale. Dropping it is what makes splitting or merging a requirement free.
 *
 * Carries no form — form is how a *test* checks, not a property of code.
 */
export function realizes(spec: string, scenario: string): void {
  void spec;
  void scenario;
}

/**
 * Marks a test as verifying a claim, at the form the test *actually* has.
 *
 * What the form must *be* lives in the verification plan; this declares what it is, and the
 * comparison between the two is `wrong-form`.
 */
export function covers(
  spec: string,
  scenario: string,
  scope: Scope,
  quantification: Quantification,
  oracle?: Oracle,
): void {
  void spec;
  void scenario;
  void scope;
  void quantification;
  void oracle;
}

/**
 * Marks a production symbol as the implementation of a design-owned mechanism identity.
 *
 * The emitter derives the symbol binding. If the symbol or marker disappears while the design
 * remains, Azimuth reports the mechanism as unresolved.
 */
export function implementsMechanism(spec: string, mechanism: string): void {
  void spec;
  void mechanism;
}

/**
 * Marks a test as evidence for a mechanism's own contract.
 *
 * This does not automatically cover every business claim that depends on the mechanism; that
 * composition needs an explicit, reviewable relation.
 */
export function coversMechanism(
  spec: string,
  mechanism: string,
  scope: Scope,
  quantification: Quantification,
  oracle?: Oracle,
): void {
  void spec;
  void mechanism;
  void scope;
  void quantification;
  void oracle;
}
