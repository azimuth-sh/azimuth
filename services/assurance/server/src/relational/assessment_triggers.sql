CREATE OR REPLACE FUNCTION assessment_record_event() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF NEW.kind IN ('run','run_attribution','context_capture','context_contract','review','authority','release','deployment') THEN
        INSERT INTO assessment_events(project,kind,record_key) VALUES(NEW.project,NEW.kind,NEW.record_key);
    END IF;
    RETURN NEW;
END $$;
DROP TRIGGER IF EXISTS assessment_record_event ON assurance_records;
CREATE TRIGGER assessment_record_event AFTER INSERT ON assurance_records FOR EACH ROW EXECUTE FUNCTION assessment_record_event();
CREATE OR REPLACE FUNCTION assessment_review_event() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    INSERT INTO assessment_events(project,kind,record_key) VALUES(NEW.project,'review',NEW.review_fingerprint);
    RETURN NEW;
END $$;
DROP TRIGGER IF EXISTS assessment_review_event ON model_revision_reviews;
CREATE TRIGGER assessment_review_event AFTER INSERT ON model_revision_reviews FOR EACH ROW EXECUTE FUNCTION assessment_review_event();
CREATE OR REPLACE FUNCTION assessment_reconciliation_event() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    INSERT INTO assessment_events(project,kind,record_key) VALUES(NEW.project,'reconciliation',NEW.environment);
    RETURN NEW;
END $$;
DROP TRIGGER IF EXISTS assessment_reconciliation_event ON deployment_observations;
CREATE TRIGGER assessment_reconciliation_event AFTER INSERT ON deployment_observations FOR EACH ROW EXECUTE FUNCTION assessment_reconciliation_event();
CREATE OR REPLACE FUNCTION assessment_release_run_event() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    INSERT INTO assessment_events(project,kind,record_key) VALUES(NEW.project,'run',NEW.run_fingerprint);
    RETURN NEW;
END $$;
DROP TRIGGER IF EXISTS assessment_release_run_event ON release_run_submissions;
CREATE TRIGGER assessment_release_run_event AFTER INSERT ON release_run_submissions FOR EACH ROW EXECUTE FUNCTION assessment_release_run_event();
CREATE OR REPLACE FUNCTION definition_fact_event() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF NEW.kind IN ('authority','review','run') THEN
        INSERT INTO definition_events(project,kind,record_key) VALUES(NEW.project,NEW.kind,NEW.record_key);
    END IF;
    RETURN NEW;
END $$;
DROP TRIGGER IF EXISTS definition_fact_event ON assurance_records;
CREATE TRIGGER definition_fact_event AFTER INSERT ON assurance_records FOR EACH ROW EXECUTE FUNCTION definition_fact_event();
CREATE OR REPLACE FUNCTION definition_association_event() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    INSERT INTO definition_events(project,kind,record_key) VALUES(NEW.project,'revision',NEW.revision_fingerprint);
    RETURN NEW;
END $$;
DROP TRIGGER IF EXISTS definition_review_event ON model_revision_reviews;
CREATE TRIGGER definition_review_event AFTER INSERT ON model_revision_reviews FOR EACH ROW EXECUTE FUNCTION definition_association_event();
DROP TRIGGER IF EXISTS definition_run_model_event ON model_revision_run_models;
CREATE TRIGGER definition_run_model_event AFTER INSERT ON model_revision_run_models FOR EACH ROW EXECUTE FUNCTION definition_association_event();
CREATE OR REPLACE FUNCTION definition_release_run_event() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    INSERT INTO definition_events(project,kind,record_key) SELECT NEW.project,'revision',model_revision
      FROM assurance_releases WHERE project=NEW.project AND fingerprint=NEW.release_fingerprint;
    RETURN NEW;
END $$;
DROP TRIGGER IF EXISTS definition_release_run_event ON release_run_submissions;
CREATE TRIGGER definition_release_run_event AFTER INSERT ON release_run_submissions FOR EACH ROW EXECUTE FUNCTION definition_release_run_event();
CREATE OR REPLACE FUNCTION invalidate_assessment_event() RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE
    check_ids text[] := '{}';
    context_ids text[] := '{}';
    capture jsonb;
    captured_subject text;
    environment_name text;
    affected_deployments text[];
    bundle text;
    review_fingerprints text[] := '{}';
    logical_run text;
BEGIN
    PERFORM pg_advisory_xact_lock(hashtextextended('assessment initialization|'||NEW.project,0));
    PERFORM 1 FROM deployment_assessments WHERE project=NEW.project ORDER BY deployment FOR UPDATE;
    IF NEW.kind IN ('run','run_attribution') THEN
        bundle := CASE WHEN NEW.kind='run_attribution' THEN left(NEW.record_key,71) ELSE NEW.record_key END;
        SELECT COALESCE(array_agg(DISTINCT c.f_id),'{}') INTO check_ids
          FROM run_bundles r JOIN run_bundles history ON history.project=r.project AND history.run_id=r.run_id
          JOIN assurance_records h ON h.kind='run' AND h.project=history.project AND h.record_key=history.fingerprint
          JOIN assurance_run_plan_checks_item c ON c._record_id=h.id
         WHERE r.project=NEW.project AND r.fingerprint=bundle;
        SELECT run_id INTO logical_run FROM run_bundles WHERE project=NEW.project AND fingerprint=bundle;
        SELECT COALESCE(array_agg(DISTINCT challenge->'target'->>'fingerprint'),'{}') INTO review_fingerprints
          FROM run_bundles history JOIN assurance_records a ON a.kind='run' AND a.project=history.project AND a.record_key=history.fingerprint CROSS JOIN LATERAL jsonb_array_elements(CASE WHEN EXISTS(SELECT 1 FROM assurance_run_challenger_executions_item target WHERE target._record_id=a.id) THEN assurance_record_field('run',history.project,history.fingerprint,'challenger_executions') ELSE '[]'::jsonb END) challenge
         WHERE history.project=NEW.project AND history.run_id=logical_run;
    ELSIF NEW.kind='context_capture' THEN
        SELECT assurance_record('context_capture',NEW.project,NEW.record_key)->'capture' INTO capture;
        IF capture->>'kind'='deployment' THEN
            captured_subject := capture->>'subject_fingerprint';
            SELECT COALESCE(array_agg(value->>'contract_revision'),'{}') INTO context_ids
              FROM jsonb_each(capture->'before'->'report'->'contexts');
        ELSE
            SELECT COALESCE(array_agg(DISTINCT c.f_id),'{}') INTO check_ids
              FROM assurance_records a JOIN assurance_run_plan_checks_item c ON c._record_id=a.id
             WHERE a.kind='run' AND a.project=NEW.project AND a.record_key=capture->>'bundle_fingerprint';
        END IF;
    ELSIF NEW.kind='context_contract' THEN
        context_ids := ARRAY[NEW.record_key];
    ELSIF NEW.kind='reconciliation' THEN
        environment_name := NEW.record_key;
    END IF;
    WITH affected AS (
        UPDATE assessment_claims c SET generation=c.generation+1
         WHERE c.project=NEW.project AND (
            NEW.kind='review'
            OR EXISTS(SELECT 1 FROM deployment_assessments h
                       WHERE h.project=c.project AND h.deployment=c.deployment
                         AND (NEW.kind='release' AND h.release=NEW.record_key
                           OR NEW.kind='authority' AND h.authority=NEW.record_key
                           OR NEW.kind='deployment' AND h.deployment=NEW.record_key))
            OR EXISTS(SELECT 1 FROM assessment_dependencies d
                       WHERE d.project=c.project AND d.deployment=c.deployment AND d.claim=c.claim
                         AND (d.kind='review' AND d.identity=ANY(review_fingerprints) OR d.kind='review-run' AND d.identity=logical_run OR d.kind='check' AND d.identity=ANY(check_ids)
                           OR d.kind='context' AND d.identity=ANY(context_ids)
                              AND (captured_subject IS NULL OR EXISTS(SELECT 1 FROM deployment_assessments h WHERE h.project=c.project AND h.deployment=c.deployment AND h.subject=captured_subject))
                           OR d.kind='environment' AND d.identity=environment_name
                              AND EXISTS(SELECT 1 FROM deployment_assessments h WHERE h.project=c.project AND h.deployment=c.deployment AND (h.observed_current OR h.deployment=(SELECT o.deployment_id FROM deployment_observations o WHERE o.project=c.project AND o.environment=environment_name ORDER BY o.revision DESC LIMIT 1))))))
         RETURNING deployment
    ) SELECT COALESCE(array_agg(DISTINCT deployment),'{}') INTO affected_deployments FROM affected;
    UPDATE deployment_assessments SET generation=generation+1,retry_at=clock_timestamp()
     WHERE project=NEW.project AND (deployment=ANY(affected_deployments) OR published_generation=0);
    UPDATE assessment_events SET invalidated=(NEW.replayed OR (current_setting('transaction_isolation')='read committed' AND EXISTS(SELECT 1 FROM deployment_assessments WHERE project=NEW.project))) WHERE sequence=NEW.sequence;
    RETURN NEW;
END $$;
DROP TRIGGER IF EXISTS invalidate_assessment_event ON assessment_events;
CREATE CONSTRAINT TRIGGER invalidate_assessment_event AFTER INSERT ON assessment_events DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION invalidate_assessment_event();
