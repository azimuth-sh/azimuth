# Local UI scenarios

From `services/assurance/web`:

```sh
npm run dev:scenarios
# http://127.0.0.1:3100/projects/payments-demo
```

Install the existing dependencies with `npm ci --ignore-scripts` if needed.
`ASSURANCE_UI_PORT=3101 npm run dev:scenarios -- incident` selects another port.
Stop with Ctrl+C. Nothing is written to PostgreSQL or production. No Docker,
OAuth application, credentials, Tailscale or API process is needed.

| Scenario | Purpose |
| --- | --- |
| `mixed` (default) | Two satisfied Requirements, one not satisfied Requirement, missing execution, stale Requirement Review, Execution Context mismatch and overdue monitoring; deployed release awaiting acceptance. |
| `healthy` | All seven Requirements satisfied; accepted deployed release. |
| `incident` | Mixed scenario plus a delivery deadline violation. |
| `empty` | Project with no model, releases, reviews or Runs. |

UI scenarios are local fixture profiles; they are unrelated to the Scenarios of a
Requirement. These are synthetic **UI read models**, not native authority projections, Run
bundles or assessor output. Never ingest them into AAS. They deliberately permit
exploring proposed experiences before the API supports them. Deployment observation freshness is provided by an explicit synthetic observation.
The read models mirror computed deployment Requirement Status: source evidence is linked through
release provenance, Execution Context mismatches remain unresolved, and Monitor
freshness is independent of deployment-observation freshness. Historical authored
assessments remain inspectable without overriding computed Status.
Dates are relative to page reads so scenarios remain
useful; they are not a real event history. Identities are deterministic synthetic
link keys, not protocol content fingerprints.

`payments.json` owns the example Requirements, Scenarios, methods, outcomes and explanations.
`src/lib/development-data.ts` adapts them to existing inspection screens. The dashboard opens the exact model and deployment referenced by the release.
The prior main model contains six reminder Requirements; the candidate adds retry
recovery as a seventh Requirement. Prior-main definitions have no synthetic reviews or
production assessments; candidate evidence is not transferred to the prior main model.
The model selector defaults to the prior main model in the mixed/incident scenario; choose
the candidate revision and production version to inspect the deployed change.
The banner remains visible throughout navigation. Pagination is intentionally one
page; unknown resources fail explicitly. This is not a backend integration or
OAuth verification environment, and it currently does not model correction chains,
concurrent releases. Production and staging are explicitly authored environments;
staging has no borrowed production assessments.

The launcher binds only to loopback, uses port 3100 by default and explicitly
enables `ASSURANCE_UI_SCENARIO`. The runtime rejects this flag outside Next.js
development mode or with a non-loopback origin. Unset the flag to use the normal
OAuth-authenticated API-backed application; unavailable API data never falls back
to scenarios. The local synthetic viewer can read only `payments-demo` and carries
no API credential or write capability.

## Color themes

The header offers System (default), Light and Dark. A one-year, same-site browser
cookie retains the selection across navigation and reloads. The server puts the
validated preference on the root HTML element before paint; System follows the OS
through CSS, including changes while the page is open. No authentication or data
configuration is needed for themes. All screen colors use semantic CSS tokens in
`src/app/globals.css`; extend both palettes when introducing a new color role.

## Model workspace

Application navigation separates Overview, Model, Releases and Records. Inside
Model, unpinned module navigation starts collapsed on every page, including after
refresh. Only an explicit Pin preference reserves an inline column. The Modules button opens a compact overlay anchored to the button, without
covering application navigation or changing the detail width; Pin keeps navigation beside the content on desktop. A one-year browser
cookie remembers the pin preference. Escape, the close button, backdrop clicks
and module selection close the overlay. The overlay adjusts its position and
height to the available viewport space. Hide and Unpin both release the inline
column immediately. On small screens navigation always uses
the overlay. Module expansion is retained during client navigation; direct Requirement
links reveal its owning module and ancestors. The tree lists modules only, with
Requirement counts; Requirement statements remain in the module/detail area. 

Model and system-version selectors use published names and descriptor labels.
Exact fingerprints and raw records live in Technical details; copy buttons use
the browser clipboard, with manual selection available if clipboard access fails.
A module view includes Requirements from its nested modules. A Requirement view separates
Requirement Status, Scenarios and declared Checks, Requirement Review, and assessment history.
Recorded Check Results must match the selected Verification Target fingerprint, current Check
fingerprint and Scenario before appearing under that Check. Displaying a Check Result
is not an applicability decision or a new Requirement Status assessment.
Check Result outcomes are labeled Passed / Failed / Inconclusive (serialized
`satisfied | violated | inconclusive`) so that they are never confused with
Requirement Status, labeled Satisfied / Not satisfied / Unresolved (serialized
`supported | violated | unresolved`).

The model workspace reads its catalog directly from the selected revision.
If that revision has no registered catalog, Requirements remain browsable under
Unassigned Requirements. The UI does not
infer a module hierarchy from identifier spelling. History and Runs are bounded
by the existing snapshot API. No acceptance or production-write controls are added.

## Domain-backed workspace fields

The synthetic profile now provides a revision catalog endpoint, an explicitly
selected current production deployment, exact deployment lookup and structured
Requirement Status explanation/coverage fields. The real UI uses the same endpoint shapes.
Production selection no longer falls back to the only recorded deployment.
The fixture's selection is explicitly authored synthetic data, not cluster
discovery. The real assessor decides accepted versus unassessed evidence; the
Requirement screen renders its coverage directly rather than joining raw Runs itself.

## Release and deployment context

The model workspace has one Browse context selector. Each release appears once and selects its model automatically. It opens the
explicitly selected production deployment when that release matches, or its sole
recorded production deployment when the full bounded list is unambiguous. Other
releases show definitions without a system assessment. The accepted-model option
remains a separate definitions view. Exact deployment links retain their context;
the browser never picks the newest deployment by assumption. Definitions have a neutral
Definition label rather than an Unresolved status. The selector never permits
an independent model/deployment pairing. Existing exact links resolve deployment
context to its release model. Candidate evidence remains separate from the
prior main model.

## Environment selection

The workspace sidebar lists recorded environment names from selections and
deployments. Selection is explicit in the URL and remembered per project in a
browser cookie. The overview, Requirement lists, release evidence and model contexts
read that environment's selection and observations. Switching environment clears
an explicitly selected deployment so evidence cannot follow the wrong environment.
The local fixture includes production and staging; staging has seven unresolved
Requirements with no execution evidence. Environment selection never deploys or updates
AAS records.

## Deployment Requirement Status

Modules organize Requirements. Scenarios describe expected behavior. Checks
produce Results. AAS assesses the evidence and shows Requirement Status for a
deployment.

Requirement details show production Monitors first, deployment Checks second, then source
and artifact verification. Each Check identifies its Verification Target, component
scope, Execution Context requirements and applicability reasons. Overdue monitoring is not
a product violation. Failed Check Results stay visible even when they cannot
establish that the Requirement is not satisfied. `npm test` verifies the actual React view
with mixed evidence, including definitions-only navigation.
