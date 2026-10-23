-- Work Provenance is a separate domain from assurance: no assessment query, trigger or event reads
-- these tables, so nothing stored here can change review currentness or Requirement Status.

CREATE TABLE provenance_ledgers (
    project text NOT NULL,
    change text NOT NULL,
    ledger_fingerprint text NOT NULL,
    -- Orders accepted submissions; the latest one is the ledger the change currently exports.
    submission bigserial NOT NULL UNIQUE,
    submitted_by text NOT NULL,
    submitted_at timestamptz NOT NULL DEFAULT now(),
    provenance_basis text NOT NULL CHECK (provenance_basis = 'publisher-transported-assertion'),
    PRIMARY KEY(project, change, ledger_fingerprint)
);

CREATE TABLE provenance_instances (
    project text NOT NULL,
    change text NOT NULL,
    instance_id text NOT NULL,
    position integer NOT NULL CHECK (position >= 0),
    -- The replaced instance as recorded by `workflow init --predecessor`; all four are null together.
    predecessor_id text,
    predecessor_ref text,
    predecessor_revision bigint,
    predecessor_fingerprint text,
    disposition text NOT NULL CHECK (disposition IN ('active', 'superseded', 'completed', 'abandoned')),
    definition_fingerprint text NOT NULL,
    -- An instance without a template records neither column.
    template_id text,
    template_revision bigint CHECK (template_revision >= 1),
    PRIMARY KEY(project, change, instance_id),
    -- A later export may insert a historical instance before existing ones, so positions are
    -- rewritten in one transaction and checked only at commit.
    UNIQUE(project, change, position) DEFERRABLE INITIALLY DEFERRED,
    CHECK ((predecessor_id IS NULL) = (predecessor_ref IS NULL)
        AND (predecessor_id IS NULL) = (predecessor_revision IS NULL)
        AND (predecessor_id IS NULL) = (predecessor_fingerprint IS NULL)),
    CHECK ((template_id IS NULL) = (template_revision IS NULL))
);

CREATE TABLE provenance_attempts (
    project text NOT NULL,
    attempt_id text NOT NULL,
    change text NOT NULL,
    instance_id text NOT NULL,
    sequence integer NOT NULL CHECK (sequence >= 1),
    action text NOT NULL,
    task text NOT NULL,
    scope text NOT NULL CHECK (scope IN ('requirement', 'check', 'verification-challenge', 'module', 'change')),
    actor text NOT NULL,
    outcome text NOT NULL,
    report text,
    handoff_captured_at_ms bigint,
    recorded_at_ms bigint,
    agent_kind text CHECK (agent_kind IN ('model', 'human')),
    agent_kind_basis text NOT NULL CHECK (agent_kind_basis IN ('harness', 'declared', 'unknown')),
    model text,
    model_basis text NOT NULL CHECK (model_basis IN ('harness', 'declared', 'unknown')),
    effort text,
    effort_basis text NOT NULL CHECK (effort_basis IN ('harness', 'declared', 'unknown')),
    harness_name text,
    harness_version text,
    harness_basis text NOT NULL CHECK (harness_basis IN ('harness', 'declared', 'unknown')),
    trace text,
    PRIMARY KEY(project, attempt_id),
    UNIQUE(project, change, instance_id, sequence),
    FOREIGN KEY(project, change, instance_id) REFERENCES provenance_instances(project, change, instance_id),
    -- An unknown descriptor field has no value and a known one always has a value.
    CHECK ((agent_kind_basis = 'unknown') = (agent_kind IS NULL)),
    CHECK ((model_basis = 'unknown') = (model IS NULL)),
    CHECK ((effort_basis = 'unknown') = (effort IS NULL)),
    CHECK ((harness_basis = 'unknown') = (harness_name IS NULL)),
    CHECK (harness_basis <> 'unknown' OR harness_version IS NULL)
);

CREATE TABLE provenance_attempt_targets (
    project text NOT NULL,
    attempt_id text NOT NULL,
    position integer NOT NULL CHECK (position >= 0),
    name text NOT NULL,
    value text NOT NULL,
    PRIMARY KEY(project, attempt_id, position),
    FOREIGN KEY(project, attempt_id) REFERENCES provenance_attempts(project, attempt_id)
);

CREATE TABLE provenance_attempt_guidance (
    project text NOT NULL,
    attempt_id text NOT NULL,
    position integer NOT NULL CHECK (position >= 0),
    kind text NOT NULL,
    guidance_id text NOT NULL,
    fingerprint text,
    PRIMARY KEY(project, attempt_id, position),
    FOREIGN KEY(project, attempt_id) REFERENCES provenance_attempts(project, attempt_id)
);

CREATE TABLE provenance_attempt_entities (
    project text NOT NULL,
    attempt_id text NOT NULL,
    role text NOT NULL CHECK (role IN ('used', 'generated')),
    position integer NOT NULL CHECK (position >= 0),
    kind text NOT NULL,
    repository text,
    ref text,
    fingerprint text,
    record text,
    -- Links a stored review to the attempt that produced it.
    record_fingerprint text,
    PRIMARY KEY(project, attempt_id, role, position),
    FOREIGN KEY(project, attempt_id) REFERENCES provenance_attempts(project, attempt_id),
    -- Only a generated entity can name a selected review-bundle entry.
    CHECK (role = 'generated' OR (record IS NULL AND record_fingerprint IS NULL))
);

CREATE TABLE provenance_failure_attributions (
    project text NOT NULL,
    attribution_id text NOT NULL,
    requirement text NOT NULL,
    adverse_kind text NOT NULL CHECK (adverse_kind IN ('requirement-status', 'check-result', 'verification-challenge-result', 'requirement-review')),
    adverse_ref text NOT NULL,
    investigator text NOT NULL,
    agent_kind text CHECK (agent_kind IN ('model', 'human')),
    agent_kind_basis text NOT NULL CHECK (agent_kind_basis IN ('harness', 'declared', 'unknown')),
    model text,
    model_basis text NOT NULL CHECK (model_basis IN ('harness', 'declared', 'unknown')),
    effort text,
    effort_basis text NOT NULL CHECK (effort_basis IN ('harness', 'declared', 'unknown')),
    harness_name text,
    harness_version text,
    harness_basis text NOT NULL CHECK (harness_basis IN ('harness', 'declared', 'unknown')),
    recorded_at_ms bigint NOT NULL,
    confidence text NOT NULL CHECK (confidence IN ('low', 'medium', 'high')),
    -- An earlier attribution may be submitted later, so revision is a recorded name, not a key.
    revises text,
    rationale text NOT NULL,
    fingerprint text NOT NULL,
    submitted_by text NOT NULL,
    submitted_at timestamptz NOT NULL DEFAULT now(),
    provenance_basis text NOT NULL CHECK (provenance_basis = 'publisher-transported-assertion'),
    PRIMARY KEY(project, attribution_id),
    CHECK ((agent_kind_basis = 'unknown') = (agent_kind IS NULL)),
    CHECK ((model_basis = 'unknown') = (model IS NULL)),
    CHECK ((effort_basis = 'unknown') = (effort IS NULL)),
    CHECK ((harness_basis = 'unknown') = (harness_name IS NULL)),
    CHECK (harness_basis <> 'unknown' OR harness_version IS NULL)
);
CREATE INDEX provenance_failure_attributions_requirement
    ON provenance_failure_attributions(project, requirement);

-- Attempts are named, not referenced: an attribution may arrive before the ledger that records them.
CREATE TABLE provenance_failure_attribution_attempts (
    project text NOT NULL,
    attribution_id text NOT NULL,
    position integer NOT NULL CHECK (position >= 0),
    attempt_id text NOT NULL,
    role text NOT NULL CHECK (role IN ('caused', 'allowed-through')),
    PRIMARY KEY(project, attribution_id, position),
    FOREIGN KEY(project, attribution_id) REFERENCES provenance_failure_attributions(project, attribution_id)
);
