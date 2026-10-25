import { shortIdentifier } from '@/lib/identifier';

export type ReviewKind = 'verification-review' | 'realization-review';
export const reviewKindNames: Record<ReviewKind, string> = {
  'verification-review': 'Verification review',
  'realization-review': 'Realization review',
};
const statusLabels: Record<string, string> = {
  accepted: 'Accepted',
  missing: 'Missing',
  stale: 'Stale',
  rejected: 'Rejected',
  invalid: 'Invalid',
  unresolved: 'Unresolved',
  'not-declared': 'Not declared',
  pending: 'Updating',
};
export function reviewLabel(status: string | undefined) {
  return status ? statusLabels[status] ?? status : 'Updating';
}
// A rejected review is the only adverse review outcome; every other undeclared or incomplete state
// asks for work without claiming the Requirement is wrong.
export function reviewTone(status: string | undefined) {
  return status === 'accepted' ? 'good' : status === 'rejected' ? 'bad' : status === 'not-declared' || status === undefined || status === 'pending' ? 'neutral' : 'warn';
}
export function declares(status: string | undefined) {
  return status !== undefined && status !== 'not-declared';
}
export function specifiedOnly(requirement: { verification_review?: string; realization_review?: string }) {
  return requirement.verification_review === 'not-declared' && requirement.realization_review === 'not-declared';
}

export function ReviewChips({ verification, realization }: { verification: string | undefined; realization: string | undefined }) {
  return <span className="reviewChips">
    <span className={`badge ${reviewTone(verification)}`}>{reviewKindNames['verification-review']} · {reviewLabel(verification)}</span>
    <span className={`badge ${reviewTone(realization)}`}>{reviewKindNames['realization-review']} · {reviewLabel(realization)}</span>
  </span>;
}

// Code drift is information about a code snapshot, never an error state, so it renders muted.
export function CodeDrift({ drift, commit }: { drift: boolean | null | undefined; commit: string | null | undefined }) {
  if (drift !== true) return null;
  return <span className="codeDrift">Reviewed at {commit ? <code title={commit}>{shortIdentifier(commit)}</code> : 'an unrecorded commit'}, code changed since</span>;
}
