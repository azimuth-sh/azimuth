CREATE TABLE deployment_observations (
    project text NOT NULL,
    environment text NOT NULL,
    revision bigint NOT NULL CHECK (revision > 0),
    fingerprint text NOT NULL,
    payload jsonb NOT NULL,
    deployment_id text,
    submitted_by text NOT NULL,
    received_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY(project,environment,revision),
    UNIQUE(project,environment,fingerprint),
    FOREIGN KEY(project,deployment_id) REFERENCES release_deployments(project,deployment_id)
);
