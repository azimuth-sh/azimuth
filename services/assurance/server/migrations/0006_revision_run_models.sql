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
