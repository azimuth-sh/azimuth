CREATE TABLE target_observations (
    sequence bigserial UNIQUE NOT NULL,
    project text NOT NULL,
    fingerprint text NOT NULL,
    source text NOT NULL,
    execution_id text NOT NULL,
    environment text NOT NULL,
    submitted_by text NOT NULL,
    received_at_ms bigint NOT NULL,
    last_resolution_attempt_ms bigint NOT NULL DEFAULT 0,
    resolution_diagnostic text NOT NULL DEFAULT 'No unambiguous complete historical reconciliation bracket and registered evidence definition',
    PRIMARY KEY(project,fingerprint),
    UNIQUE(project,source,execution_id)
);
CREATE TABLE target_observation_resolutions (
    project text NOT NULL,
    observation_fingerprint text NOT NULL,
    deployment_id text NOT NULL,
    verification_target_fingerprint text NOT NULL,
    before_fingerprint text NOT NULL,
    after_fingerprint text NOT NULL,
    run_fingerprint text,
    capture_fingerprint text,
    normalizer_fingerprint text NOT NULL,
    resolved_at_ms bigint NOT NULL,
    PRIMARY KEY(project,observation_fingerprint),
    FOREIGN KEY(project,observation_fingerprint) REFERENCES target_observations(project,fingerprint),
    FOREIGN KEY(project,deployment_id) REFERENCES release_deployments(project,deployment_id)
);
CREATE INDEX target_observations_pending ON target_observations(project,sequence);
