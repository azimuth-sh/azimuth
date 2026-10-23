-- Expand-only: rows written before this migration become revision 1 with no recorded sources,
-- and the previous release keeps working against the widened tables.
ALTER TABLE main_publications
    ADD COLUMN publication_revision integer NOT NULL DEFAULT 1 CHECK (publication_revision >= 1);

-- The relational record store keys the document by this value; revision 1 keeps the bare commit so
-- records written before revisions existed stay addressable.
CREATE FUNCTION main_publication_record_key(c text,r integer) RETURNS text
LANGUAGE sql IMMUTABLE AS $$
    SELECT CASE WHEN r = 1 THEN c ELSE c || ':' || r END
$$;

ALTER TABLE main_publication_reviews
    ADD COLUMN publication_revision integer NOT NULL DEFAULT 1 CHECK (publication_revision >= 1);

-- Constraint names differ between PostgreSQL versions, so they are found rather than named.
DO $$
DECLARE c record;
BEGIN
    FOR c IN
        SELECT conrelid::regclass AS relation, conname
        FROM pg_constraint
        WHERE (conrelid = 'main_publication_reviews'::regclass AND contype IN ('f','p'))
           OR (conrelid = 'main_publications'::regclass AND contype IN ('p','u'))
        ORDER BY conrelid = 'main_publication_reviews'::regclass DESC
    LOOP
        EXECUTE format('ALTER TABLE %s DROP CONSTRAINT %I', c.relation, c.conname);
    END LOOP;
END $$;

ALTER TABLE main_publications ADD PRIMARY KEY(project,commit_sha,publication_revision);
ALTER TABLE main_publication_reviews
    ADD PRIMARY KEY(project,commit_sha,publication_revision,review_fingerprint),
    ADD FOREIGN KEY(project,commit_sha,publication_revision)
        REFERENCES main_publications(project,commit_sha,publication_revision);

-- Rows rather than a json column: migrate() treats any json/jsonb column as a leftover document
-- store and refuses to start. A publication with no rows here predates recorded sources.
CREATE TABLE main_publication_sources (
    project text NOT NULL,
    commit_sha text NOT NULL,
    publication_revision integer NOT NULL,
    repository text NOT NULL CHECK (btrim(repository) <> ''),
    source_commit text NOT NULL CHECK (source_commit ~ '^[0-9a-f]{40}$'),
    PRIMARY KEY(project,commit_sha,publication_revision,repository),
    FOREIGN KEY(project,commit_sha,publication_revision)
        REFERENCES main_publications(project,commit_sha,publication_revision)
);

CREATE FUNCTION main_publication_sources(p text,c text,r integer) RETURNS jsonb
LANGUAGE sql STABLE AS $$
    SELECT COALESCE(jsonb_object_agg(repository,source_commit),'{}'::jsonb)
    FROM main_publication_sources
    WHERE project=p AND commit_sha=c AND publication_revision=r
$$;

DROP INDEX main_publications_latest;
CREATE INDEX main_publications_latest
    ON main_publications(project,main_sequence DESC,publication_revision DESC);
