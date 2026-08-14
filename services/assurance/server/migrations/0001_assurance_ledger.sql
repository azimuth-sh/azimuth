CREATE TABLE projects (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    created_at BIGINT NOT NULL,
    content_fingerprint TEXT NOT NULL
);

CREATE TABLE project_model_snapshots (
    sequence BIGSERIAL NOT NULL UNIQUE,
    project_id TEXT NOT NULL REFERENCES projects(id),
    id TEXT NOT NULL,
    model_fingerprint TEXT NOT NULL,
    content_fingerprint TEXT NOT NULL,
    payload JSONB NOT NULL,
    PRIMARY KEY (project_id, id)
);

CREATE INDEX ix_project_model_snapshots_history
    ON project_model_snapshots(project_id, sequence DESC);

CREATE TABLE evidence_definitions (
    project_id TEXT NOT NULL REFERENCES projects(id),
    logical_id TEXT NOT NULL,
    definition_fingerprint TEXT NOT NULL,
    declared_at BIGINT NOT NULL,
    content_fingerprint TEXT NOT NULL,
    payload JSONB NOT NULL,
    PRIMARY KEY (project_id, logical_id, definition_fingerprint)
);

CREATE INDEX ix_evidence_definitions_current
    ON evidence_definitions(project_id, logical_id, declared_at DESC);

CREATE TABLE qualifications (
    project_id TEXT NOT NULL REFERENCES projects(id),
    id TEXT NOT NULL,
    definition_id TEXT NOT NULL,
    definition_fingerprint TEXT NOT NULL,
    qualified_at BIGINT NOT NULL,
    content_fingerprint TEXT NOT NULL,
    payload JSONB NOT NULL,
    PRIMARY KEY (project_id, id)
);

CREATE TABLE observations (
    project_id TEXT NOT NULL REFERENCES projects(id),
    id TEXT NOT NULL,
    definition_id TEXT NOT NULL,
    definition_fingerprint TEXT NOT NULL,
    observed_at BIGINT NOT NULL,
    content_fingerprint TEXT NOT NULL,
    payload JSONB NOT NULL,
    PRIMARY KEY (project_id, id)
);

CREATE TABLE challenges (
    project_id TEXT NOT NULL REFERENCES projects(id),
    id TEXT NOT NULL,
    source TEXT NOT NULL,
    definition_id TEXT NOT NULL,
    definition_fingerprint TEXT NOT NULL,
    observed_at BIGINT NOT NULL,
    content_fingerprint TEXT NOT NULL,
    payload JSONB NOT NULL,
    PRIMARY KEY (project_id, id)
);

CREATE TABLE gate_decisions (
    sequence BIGSERIAL NOT NULL UNIQUE,
    project_id TEXT NOT NULL REFERENCES projects(id),
    id UUID NOT NULL,
    definition_id TEXT NOT NULL,
    stage TEXT NOT NULL,
    target_fingerprint TEXT NOT NULL,
    evaluated_at BIGINT NOT NULL,
    request JSONB NOT NULL,
    decision JSONB NOT NULL,
    PRIMARY KEY (project_id, id)
);

CREATE INDEX ix_gate_decisions_history
    ON gate_decisions(project_id, sequence DESC);

CREATE INDEX ix_gate_decisions_current_target
    ON gate_decisions(project_id, target_fingerprint, sequence DESC);
