CREATE TABLE run_bundles (
    project TEXT NOT NULL,
    run_id TEXT NOT NULL,
    revision BIGINT NOT NULL CHECK (revision >= 0),
    fingerprint TEXT NOT NULL,
    subject_fingerprint TEXT NOT NULL,
    model_fingerprint TEXT NOT NULL,
    payload JSONB NOT NULL,
    ingested_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (project, run_id, revision),
    UNIQUE (project, fingerprint)
);

CREATE INDEX ix_run_bundles_subject ON run_bundles(project, subject_fingerprint, run_id, revision DESC);
