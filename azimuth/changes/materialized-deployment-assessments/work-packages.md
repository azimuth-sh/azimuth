# Work packages

## Work package: assessment-service
Status: in-progress
Depends on: none
Owns: services/assurance/server, tools/azimuth/src/deployment_status.rs
Objective: Durable relational materialization and selective native Claim recomputation; generation-safe event invalidation, expiry, worker recovery and fast compact/detailed read APIs.

## Work package: assessment-interface
Status: complete
Depends on: none
Owns: services/assurance/web
Objective: Consume compact assessment overview with explicit freshness and pending states; preserve detail screens and eliminate avoidable sequential requests.

UI evidence: TypeScript checks, production Next.js build, Claim verification rendering, compact overview and model freshness rendering checks pass locally. Independent review finding for missing freshness metadata corrected and covered.
