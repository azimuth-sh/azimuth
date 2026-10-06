CREATE TABLE assurance_authorities (
    project text NOT NULL,
    fingerprint text NOT NULL,
    payload jsonb NOT NULL,
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
    payload jsonb NOT NULL,
    submitted_by text NOT NULL,
    submitted_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY(project, fingerprint)
);
CREATE INDEX assurance_reviews_target ON assurance_reviews(project, target_fingerprint);
