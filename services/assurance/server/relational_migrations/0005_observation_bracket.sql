ALTER TABLE target_observations ADD COLUMN started_at_ms bigint, ADD COLUMN finished_at_ms bigint;
CREATE INDEX deployment_observations_environment_time ON deployment_observations(project,environment,observed_at_ms);
