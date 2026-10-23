# Change map: Change tracking model

Candidate changes only. None is approved and each needs its own proposal. Ordering reflects dependencies, not priority.

| Candidate | Scope | Depends on | Notes |
|---|---|---|---|
| Fingerprint scheme version | extractor manifest records, core validation | none | Smallest change; makes every later migration cleaner. Decide the field before any other fingerprint change. |
| Helm rendered-resource fingerprint | `tools/extractors/helm` | scheme version | Hash identity plus canonical rendered resource; drop raw template, default values and Helm version. Annotation marker form to be decided. |
| Python Check hashing at function level | `tools/extractors/python` | scheme version | Replaces whole-module bytes for Check implementations. |
| .NET Check support narrowing | `tools/extractors/dotnet` | scheme version, declared test-side dependencies | Method plus declared helpers instead of every file of the type. |
| Review basis limited to definitions and Check implementations | core assurance review, `contracts/assurance-reviews.md` | open questions 1 to 3 | The central model decision; removes production-side fingerprints from the review packet and keeps tags as an index. |
| Carry-forward migration | service ingestion, assessor, migration tool, contract | scheme version | Reusable for every later scheme change; first customer would be the batch above. |
| Reviewed-basis retention and delta view | review records, service, UI | none | Cannot be backfilled; decide before more reviews accumulate. |
| Executed-sites in Runs | Run protocol, adapters | none | Serves agent context and test selection; optional coverage-derived unit inputs. |
| Map completeness lint | traceability report or Verification Challenge | none | Flags relevant untagged files and rendered resources. |
| Agent context brief | CLI or service read, web action | executed-sites, completeness lint | Assembles Requirement, failing Check, Run diagnostics, sites, design excerpts and the release diff. |
| Normalized token or syntax-tree hashing | extractors | scheme version, carry-forward | Defer until repeated non-semantic re-reviews justify it. |

Suggested first batch if work resumes: scheme version, Helm, Python and .NET narrowing, folded into a re-review that is already required. Keep the carry-forward design open until open question 6 is answered.
