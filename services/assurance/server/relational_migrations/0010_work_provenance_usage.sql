-- Attempts and attributions stored under 0009 predate these fields; the defaults read them back as an
-- unknown agent type and unknown usage, which is what a re-export of the same ledger records.

ALTER TABLE provenance_attempts
    ADD COLUMN agent_type text,
    ADD COLUMN agent_type_basis text NOT NULL DEFAULT 'unknown' CHECK (agent_type_basis IN ('harness', 'declared', 'unknown')),
    ADD CHECK ((agent_type_basis = 'unknown') = (agent_type IS NULL)),
    -- Usage is either wholly unknown (every column null) or recorded with every required count.
    ADD COLUMN usage_basis text CHECK (usage_basis IN ('harness', 'declared')),
    ADD COLUMN model_calls bigint CHECK (model_calls >= 0),
    ADD COLUMN tool_calls bigint CHECK (tool_calls >= 0),
    ADD COLUMN input_tokens bigint CHECK (input_tokens >= 0),
    ADD COLUMN cache_read_input_tokens bigint CHECK (cache_read_input_tokens >= 0),
    ADD COLUMN cache_write_input_tokens bigint CHECK (cache_write_input_tokens >= 0),
    ADD COLUMN output_tokens bigint CHECK (output_tokens >= 0),
    ADD COLUMN reasoning_output_tokens bigint CHECK (reasoning_output_tokens >= 0),
    ADD COLUMN usage_overlapping boolean,
    ADD COLUMN delegated_agents bigint CHECK (delegated_agents >= 0),
    ADD COLUMN delegated_model_calls bigint CHECK (delegated_model_calls >= 0),
    ADD COLUMN delegated_input_tokens bigint CHECK (delegated_input_tokens >= 0),
    ADD COLUMN delegated_cache_read_input_tokens bigint CHECK (delegated_cache_read_input_tokens >= 0),
    ADD COLUMN delegated_cache_write_input_tokens bigint CHECK (delegated_cache_write_input_tokens >= 0),
    ADD COLUMN delegated_output_tokens bigint CHECK (delegated_output_tokens >= 0),
    ADD COLUMN delegated_reasoning_output_tokens bigint CHECK (delegated_reasoning_output_tokens >= 0),
    ADD CHECK ((usage_basis IS NULL) = (model_calls IS NULL)
        AND (usage_basis IS NULL) = (tool_calls IS NULL)
        AND (usage_basis IS NULL) = (input_tokens IS NULL)
        AND (usage_basis IS NULL) = (output_tokens IS NULL)
        AND (usage_basis IS NULL) = (usage_overlapping IS NULL)
        AND (usage_basis IS NULL) = (delegated_agents IS NULL)
        AND (usage_basis IS NULL) = (delegated_model_calls IS NULL)
        AND (usage_basis IS NULL) = (delegated_input_tokens IS NULL)
        AND (usage_basis IS NULL) = (delegated_output_tokens IS NULL)),
    -- Optional token counts exist only inside recorded usage.
    ADD CHECK (usage_basis IS NOT NULL OR (cache_read_input_tokens IS NULL
        AND cache_write_input_tokens IS NULL
        AND reasoning_output_tokens IS NULL
        AND delegated_cache_read_input_tokens IS NULL
        AND delegated_cache_write_input_tokens IS NULL
        AND delegated_reasoning_output_tokens IS NULL));

ALTER TABLE provenance_failure_attributions
    ADD COLUMN agent_type text,
    ADD COLUMN agent_type_basis text NOT NULL DEFAULT 'unknown' CHECK (agent_type_basis IN ('harness', 'declared', 'unknown')),
    ADD CHECK ((agent_type_basis = 'unknown') = (agent_type IS NULL));
