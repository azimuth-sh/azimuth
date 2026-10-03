/**
 * Azimuth linkage tags for TypeScript.
 *
 * The front end is functions — route handlers, server components, hooks — not classes, so the
 * tags are typed no-op *function calls* rather than decorators, which are class-member-only. They
 * exist to be type-checked at author time and found statically by the emitter, which resolves each
 * call's enclosing named symbol as the site. At runtime they do nothing.
 */

/**
 * Marks a production-code site as being on a claim's path, keyed by its stable project-wide Claim ID.
 *
 * The ID identifies the independently governed Claim. Cases remain repository-owned evidence
 * addresses and never enter source markers.
 *
 * Carries no form — form is how a *test* checks, not a property of code.
 */
export function realizes(claim: string): void {
  void claim;
}

/**
 * Marks a source site as an implementation of one stable project-wide Check identity.
 *
 * Claim linkage, evidence form and Qualification meaning remain repository declarations. The
 * marker supplies implementation identity only.
 */
export function implementsCheck(check: string): void {
  void check;
}

/**
 * Marks a production symbol as the implementation of a design-owned mechanism identity.
 *
 * The emitter derives the symbol binding. If the symbol or marker disappears while the design
 * remains, Azimuth reports the mechanism as unresolved.
 */
export function implementsMechanism(mechanism: string): void {
  void mechanism;
}
