CREATE TABLE assessment_events (
    sequence bigserial PRIMARY KEY, project text NOT NULL, kind text NOT NULL, record_key text NOT NULL, invalidated boolean NOT NULL DEFAULT false, replayed boolean NOT NULL DEFAULT false
);
CREATE INDEX assessment_events_project ON assessment_events(project,sequence);
CREATE TABLE deployment_assessments (
    project text NOT NULL, deployment text NOT NULL, release text NOT NULL,
    authority text, model text, verification_target text,
    generation bigint NOT NULL DEFAULT 1, published_generation bigint NOT NULL DEFAULT 0,
    evaluated_at_ms bigint, valid_until_ms bigint,
    observed_current boolean NOT NULL DEFAULT false,
    last_error text, retry_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY(project,deployment)
);
CREATE TABLE assessment_requirements (
    project text NOT NULL, deployment text NOT NULL, requirement text NOT NULL,
    generation bigint NOT NULL DEFAULT 1, published_generation bigint NOT NULL DEFAULT 0,
    record_key text, status text NOT NULL DEFAULT 'unresolved', review text NOT NULL DEFAULT 'missing',
    summary text NOT NULL DEFAULT 'Assessment is pending', evaluated_at_ms bigint, valid_until_ms bigint,
    PRIMARY KEY(project,deployment,requirement),
    FOREIGN KEY(project,deployment) REFERENCES deployment_assessments(project,deployment)
);
CREATE TABLE assessment_dependencies (
    project text NOT NULL, deployment text NOT NULL, requirement text NOT NULL,
    kind text NOT NULL, identity text NOT NULL,
    PRIMARY KEY(project,deployment,requirement,kind,identity),
    FOREIGN KEY(project,deployment,requirement) REFERENCES assessment_requirements(project,deployment,requirement)
);
CREATE INDEX assessment_dependency_lookup ON assessment_dependencies(project,kind,identity);
CREATE TABLE assessment_scenarios (
    project text NOT NULL, deployment text NOT NULL, requirement text NOT NULL,
    scenario_id text NOT NULL, check_id text NOT NULL, status text NOT NULL, generation bigint NOT NULL, result_fingerprint text NOT NULL,
    PRIMARY KEY(project,deployment,requirement,scenario_id,check_id),
    FOREIGN KEY(project,deployment,requirement) REFERENCES assessment_requirements(project,deployment,requirement)
);

CREATE INDEX assessment_requirement_expiry ON assessment_requirements(valid_until_ms) WHERE generation=published_generation;
CREATE INDEX assessment_jobs ON deployment_assessments(retry_at) WHERE generation<>published_generation OR authority IS NULL;

CREATE TABLE definition_assessments (
    project text NOT NULL, revision text NOT NULL, generation bigint NOT NULL DEFAULT 1,
    published_generation bigint NOT NULL DEFAULT 0, evaluated_at_ms bigint,
    retry_at timestamptz NOT NULL DEFAULT now(), last_error text,
    PRIMARY KEY(project,revision)
);
CREATE TABLE definition_requirements (
    project text NOT NULL, revision text NOT NULL, requirement text NOT NULL,
    record_fingerprint text, status text NOT NULL, diagnostics text[] NOT NULL,
    PRIMARY KEY(project,revision,requirement),
    FOREIGN KEY(project,revision) REFERENCES definition_assessments(project,revision)
);
CREATE TABLE definition_run_dependencies (
    project text NOT NULL, revision text NOT NULL, run text NOT NULL,
    PRIMARY KEY(project,revision,run),
    FOREIGN KEY(project,revision) REFERENCES definition_assessments(project,revision)
);
CREATE TABLE definition_events (
    sequence bigserial PRIMARY KEY, project text NOT NULL, kind text NOT NULL, record_key text NOT NULL
);
CREATE INDEX definition_events_project ON definition_events(project,sequence);
