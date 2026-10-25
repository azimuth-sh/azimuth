-- Storage version 10 is reached only by resetting the database. `kind` has no default, so a database
-- still holding version 9 definition rows fails here instead of guessing which review each row meant.

-- A Requirement carries two independent review dimensions, so definition applicability is kept per
-- review kind; code drift is informational state of a Realization Review head, never a status.
ALTER TABLE definition_requirements
    ADD COLUMN kind text NOT NULL CHECK (kind IN ('verification-review', 'realization-review')),
    ADD COLUMN code_drift boolean,
    ADD CHECK (kind = 'realization-review' OR code_drift IS NULL),
    DROP CONSTRAINT definition_requirements_pkey,
    ADD PRIMARY KEY (project, revision, requirement, kind);

ALTER TABLE assessment_requirements
    DROP COLUMN review,
    ADD COLUMN verification_review text NOT NULL DEFAULT 'missing',
    ADD COLUMN realization_review text NOT NULL DEFAULT 'missing',
    ADD COLUMN realization_code_drift boolean,
    -- Overview lists name the commit a drifted Realization Review looked at without reading the
    -- full explanation document of every Requirement.
    ADD COLUMN realization_reviewed_commit text;

-- A Realization Review is an opinion about a code snapshot, not evidence, so it is never the adverse
-- fact a failure attribution explains.
ALTER TABLE provenance_failure_attributions
    DROP CONSTRAINT provenance_failure_attributions_adverse_kind_check,
    ADD CHECK (adverse_kind IN ('requirement-status', 'check-result', 'verification-challenge-result', 'verification-review'));
