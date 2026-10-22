DROP TABLE release_acceptances;
DROP TABLE authority_selections;
DROP TABLE current_authorities;

ALTER TABLE assurance_releases
    DROP COLUMN baseline_model_revision,
    ADD COLUMN integration_commit text CHECK (integration_commit ~ '^([0-9a-f]{40}|[0-9a-f]{64})$'),
    ADD COLUMN integration_sequence bigint CHECK (integration_sequence >= 0);

CREATE TABLE main_publications (
    project text NOT NULL,
    commit_sha text NOT NULL CHECK (commit_sha ~ '^([0-9a-f]{40}|[0-9a-f]{64})$'),
    main_sequence bigint NOT NULL CHECK (main_sequence >= 1),
    model_revision text NOT NULL,
    ratification_status text NOT NULL CHECK (ratification_status IN ('ratified','unratified')),
    ratification_verified_at_ms bigint NOT NULL,
    published_by text NOT NULL,
    published_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY(project,commit_sha),
    UNIQUE(project,main_sequence),
    FOREIGN KEY(project,model_revision) REFERENCES model_revisions(project,fingerprint)
);
CREATE INDEX main_publications_latest ON main_publications(project,main_sequence DESC);

CREATE TABLE main_publication_reviews (
    project text NOT NULL,
    commit_sha text NOT NULL,
    review_fingerprint text NOT NULL,
    assessment text NOT NULL,
    diagnostics text[] NOT NULL,
    PRIMARY KEY(project,commit_sha,review_fingerprint),
    FOREIGN KEY(project,commit_sha) REFERENCES main_publications(project,commit_sha),
    FOREIGN KEY(project,review_fingerprint) REFERENCES assurance_reviews(project,fingerprint)
);

ALTER TABLE assessment_requirements ADD COLUMN supported_since_ms bigint;
