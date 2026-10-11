CREATE TABLE model_revisions (
    project text NOT NULL,
    fingerprint text NOT NULL,
    registration_status text NOT NULL DEFAULT 'candidate' CHECK (registration_status = 'candidate'),
    registered_by text NOT NULL,
    registered_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY(project, fingerprint),
    FOREIGN KEY(project, fingerprint) REFERENCES assurance_authorities(project, fingerprint)
);

INSERT INTO model_revisions(project,fingerprint,registered_by,registered_at)
SELECT project,fingerprint,registered_by,registered_at FROM assurance_authorities;

CREATE TABLE model_revision_reviews (
    project text NOT NULL,
    revision_fingerprint text NOT NULL,
    review_fingerprint text NOT NULL,
    linkage_kind text NOT NULL DEFAULT 'reviewer-submission' CHECK (linkage_kind IN ('reviewer-submission','migration-selected-authority')),
    submitted_by text NOT NULL,
    submitted_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY(project,revision_fingerprint,review_fingerprint),
    FOREIGN KEY(project,revision_fingerprint) REFERENCES model_revisions(project,fingerprint),
    FOREIGN KEY(project,review_fingerprint) REFERENCES assurance_reviews(project,fingerprint)
);

-- Existing reviews retain their assessment context at migration; historic validity is still assessed from exact dependencies.
INSERT INTO model_revision_reviews(project,revision_fingerprint,review_fingerprint,linkage_kind,submitted_by,submitted_at)
SELECT r.project,c.fingerprint,r.fingerprint,'migration-selected-authority',r.submitted_by,r.submitted_at
FROM assurance_reviews r JOIN current_authorities c ON r.project=c.project;
