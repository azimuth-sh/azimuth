CREATE TABLE run_bundles (
    project TEXT NOT NULL,
    run_id TEXT NOT NULL,
    revision BIGINT NOT NULL CHECK (revision >= 0),
    fingerprint TEXT NOT NULL,
    verification_target_fingerprint TEXT NOT NULL,
    model_fingerprint TEXT NOT NULL,
    ingested_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (project, run_id, revision),
    UNIQUE (project, fingerprint)
);

CREATE INDEX ix_run_bundles_verification_target ON run_bundles(project, verification_target_fingerprint, run_id, revision DESC);

CREATE TABLE assurance_authorities (
    project text NOT NULL,
    fingerprint text NOT NULL,
    registered_by text NOT NULL,
    registered_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY(project, fingerprint)
);
CREATE TABLE current_authorities (
    project text PRIMARY KEY,
    fingerprint text NOT NULL,
    FOREIGN KEY(project, fingerprint) REFERENCES assurance_authorities(project, fingerprint)
);
CREATE TABLE authority_selections (
    sequence bigserial PRIMARY KEY,
    project text NOT NULL,
    fingerprint text NOT NULL,
    selected_by text NOT NULL,
    selected_at timestamptz NOT NULL DEFAULT now(),
    FOREIGN KEY(project, fingerprint) REFERENCES assurance_authorities(project, fingerprint)
);
CREATE TABLE assurance_reviews (
    project text NOT NULL,
    fingerprint text NOT NULL,
    target_fingerprint text NOT NULL,
    submitted_by text NOT NULL,
    submitted_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY(project, fingerprint)
);
CREATE INDEX assurance_reviews_target ON assurance_reviews(project, target_fingerprint);

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

CREATE TABLE assurance_releases (
    project text NOT NULL,
    fingerprint text NOT NULL,
    model_revision text NOT NULL,
    label text NOT NULL,
    baseline_model_revision text NOT NULL,
    submitted_by text NOT NULL,
    submitted_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY(project,fingerprint),
    FOREIGN KEY(project,model_revision) REFERENCES model_revisions(project,fingerprint)
);
CREATE TABLE release_changes (
    project text NOT NULL,
    release_fingerprint text NOT NULL,
    change_id text NOT NULL,
    change_revision text NOT NULL,
    PRIMARY KEY(project,release_fingerprint,change_id),
    FOREIGN KEY(project,release_fingerprint) REFERENCES assurance_releases(project,fingerprint)
);
CREATE TABLE release_deployments (
    project text NOT NULL,
    deployment_id text NOT NULL,
    release_fingerprint text NOT NULL,
    submitted_by text NOT NULL,
    submitted_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY(project,deployment_id),
    FOREIGN KEY(project,release_fingerprint) REFERENCES assurance_releases(project,fingerprint)
);
CREATE TABLE release_acceptances (
    project text NOT NULL,
    release_fingerprint text NOT NULL,
    previous_model_revision text NOT NULL,
    reason text NOT NULL,
    accepted_by text NOT NULL,
    accepted_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY(project,release_fingerprint),
    FOREIGN KEY(project,release_fingerprint) REFERENCES assurance_releases(project,fingerprint)
);
ALTER TABLE assurance_reviews ADD COLUMN provenance_basis text NOT NULL DEFAULT 'authenticated-reviewer'
    CHECK(provenance_basis IN ('authenticated-reviewer','publisher-transported-assertion'));
CREATE TABLE release_run_submissions (
    project text NOT NULL,
    release_fingerprint text NOT NULL,
    run_fingerprint text NOT NULL,
    submitted_by text NOT NULL,
    PRIMARY KEY(project,release_fingerprint,run_fingerprint),
    FOREIGN KEY(project,release_fingerprint) REFERENCES assurance_releases(project,fingerprint)
);

CREATE TABLE model_revision_run_models (
    project text NOT NULL,
    revision_fingerprint text NOT NULL,
    run_model_fingerprint text NOT NULL,
    representative_run_fingerprint text NOT NULL,
    reason text NOT NULL CHECK (length(trim(reason)) BETWEEN 1 AND 16384),
    associated_by text NOT NULL,
    associated_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY(project, revision_fingerprint, run_model_fingerprint),
    FOREIGN KEY(project, revision_fingerprint) REFERENCES model_revisions(project, fingerprint),
    FOREIGN KEY(project, representative_run_fingerprint) REFERENCES run_bundles(project, fingerprint)
);

CREATE TABLE model_revision_catalogs (
    project text NOT NULL,
    revision_fingerprint text NOT NULL,
    catalog_fingerprint text NOT NULL,
    submitted_by text NOT NULL,
    provenance_basis text NOT NULL CHECK (provenance_basis IN ('owner-registration', 'publisher-publication')),
    submitted_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY(project,revision_fingerprint),
    FOREIGN KEY(project,revision_fingerprint) REFERENCES model_revisions(project,fingerprint)
);

CREATE TABLE environment_deployment_selections (
    project text NOT NULL,
    environment text NOT NULL,
    revision bigint NOT NULL CHECK (revision > 0),
    deployment_id text NOT NULL,
    reason text NOT NULL,
    selected_by text NOT NULL,
    selected_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY(project,environment,revision),
    FOREIGN KEY(project,deployment_id) REFERENCES release_deployments(project,deployment_id)
);

CREATE TABLE deployment_observations (
    project text NOT NULL,
    environment text NOT NULL,
    revision bigint NOT NULL CHECK (revision > 0),
    fingerprint text NOT NULL,
    observed_at_ms bigint NOT NULL,
    valid_for_seconds bigint NOT NULL,
    state text NOT NULL,
    release_fingerprint text,
    deployment_id text,
    submitted_by text NOT NULL,
    received_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY(project,environment,revision),
    UNIQUE(project,environment,fingerprint),
    FOREIGN KEY(project,deployment_id) REFERENCES release_deployments(project,deployment_id)
);
