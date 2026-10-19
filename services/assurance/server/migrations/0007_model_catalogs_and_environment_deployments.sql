CREATE TABLE model_revision_catalogs (
    project text NOT NULL,
    revision_fingerprint text NOT NULL,
    catalog_fingerprint text NOT NULL,
    payload jsonb NOT NULL,
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
