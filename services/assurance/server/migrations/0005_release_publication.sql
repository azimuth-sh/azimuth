CREATE TABLE assurance_releases (
    project text NOT NULL,
    fingerprint text NOT NULL,
    model_revision text NOT NULL,
    payload jsonb NOT NULL,
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
    payload jsonb NOT NULL,
    PRIMARY KEY(project,release_fingerprint,change_id),
    FOREIGN KEY(project,release_fingerprint) REFERENCES assurance_releases(project,fingerprint)
);
CREATE TABLE release_deployments (
    project text NOT NULL,
    deployment_id text NOT NULL,
    release_fingerprint text NOT NULL,
    payload jsonb NOT NULL,
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
