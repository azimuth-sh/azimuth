# After the first use of increment 2: behaviors

This is the list of behaviors for a short pass between the second and the third increment of the core. The first real change that was made with increment 2 added rules to `docs/design.md` that the shipped skill does not say yet, and it asked for one more list in `azimuth check show`. A person reads this list before the build starts.

Each row is one observable fact. The second cell is copied exactly from `docs/design.md`. A plain test fails when a quoted sentence is no longer in that file. A behavior with no sentence to quote is not built.

Status of this list: read and accepted by a person on 2026-11-20, 38 rows first and eleven more after five decisions of the same day.

## The skill: finished folders

Step 1 of the shipped skill already tells the agent to move every change folder that exists on the target branch into `azimuth/changes/finished/`.

| Behavior | Quote from the design |
| --- | --- |
| Step 1 names the moved folder `<date>-<name>` | The moved folder is named `<date>-<name>`. |
| It says which day the date is: the day on which the folder arrived on the target branch | The date is the day on which the folder arrived on the target branch. |
| It gives one git command that prints that day in UTC, and tells the agent not to type the date | The process reads it from git, in UTC |
| For a folder that arrived by a merge, the command of the skill prints the day of the merge on the target branch, and not the day of the commit on the change branch | The date is the day on which the folder arrived on the target branch. |
| Run on two branches that start from the same target branch, the command prints the same day | both made the same rename, because both read the same date from git and nobody types it |

## The skill: Scenarios

| Behavior | Quote from the design |
| --- | --- |
| Step 2 says that a Requirement and its Scenarios speak about the system as somebody outside it meets it | A Requirement and its Scenarios speak about the system as somebody outside it meets it: a learner, an admin, an operator, another system |

## The skill: the parts of a Check

| Behavior | Quote from the design |
| --- | --- |
| Step 2 says that a Check may be made of a test through a door of the system and of tests of one part in isolation | A Check may be made of parts: a test through a door of the system, and tests of one part in isolation. |
| It says that every part carries the marker of the Check | All of them carry the marker of the Check |
| It says that a Check is never made only of tests of one part in isolation | A Check is never made only of tests of one part in isolation. |
| It gives the order of the claim of such a Check: the door, what each part shows, and last what is not shown | Its order is fixed: through which door the test looks, what each part shows, and last what is not shown. |
| It says that a journey, one test through several real operations, may be a part of several Checks and carries the marker of each Check that relies on it | It carries the marker of every Check that relies on it. |
| It says that the claim of each such Check names the stretch of the journey that it relies on | The claim of each such Check names the stretch of the journey that it relies on |
| It says that a Check also has a test of one operation alone, where that operation can be tested alone | Where one operation can be tested alone, the Check also has a test of that operation alone |

## The skill: the plan

| Behavior | Quote from the design |
| --- | --- |
| Step 2 says that the change needs a plan when a Requirement that it adds or changes declares `Realization:` | A change needs a plan when a Requirement that it adds or changes declares `Realization:` |
| It says that a change with no such Requirement needs no plan | A change with no such Requirement needs no plan |
| It puts the writing of the plan after the Scenarios and the Checks, and before the one stop of the step | The agent writes the plan in the Intent step, after the Scenarios and the Checks, and the person reads all three at the one stop |
| It names what the plan holds at the stop: the proposed decisions, the units to add, change or delete, and the migration steps | At that stop the plan holds the proposed decisions, the units to add, change or delete, and the migration steps |
| It says that what somebody outside the system can observe is a Scenario and no line of the plan, and that the addresses and the answers between the team's own components are in the plan | What somebody outside the system can observe is not in the plan. It is a Scenario. The addresses and the answers between the team's own components are in the plan |
| It asks about data and the release before | What data is added or changed, and how does the release before it run beside it? |
| It asks about the promises between components | What do the team's own components promise each other? |
| It asks who may do it | Who may do it, and where is that enforced? |
| It asks how a refusal reaches the caller | How does a refusal reach the caller? |
| It asks about two requests that meet | What happens when two requests meet? |
| It asks what can no longer be deleted | What can no longer be deleted? |
| It says that a question with an answer becomes a proposed decision, and that a question with no answer leaves no line | A question with an answer becomes a proposed decision in the plan. A question with no answer leaves no line. |
| The stop of step 2 names the plan among what the person reads | The person reads the plan at the Intent stop. |
| It names the file of the plan, `azimuth/changes/<name>/plan.md` | The plan is the file `plan.md` in the folder of its change. |

## The skill: the Build and the hand over

| Behavior | Quote from the design |
| --- | --- |
| Step 3 tells the agent to move a proposed decision into `realization.md` only when the finished code follows it | It moves to `realization.md` only if the finished code follows it. |
| Step 4 tells the agent to name every proposed decision that the finished code does not follow, and to say why | At the hand over the agent names every proposed decision that the finished code does not follow, and says why |

## `azimuth check show`

| Behavior | Quote from the design |
| --- | --- |
| `azimuth check show <id>` prints the marker lines of the Check, each with its file and line | The marker lines, each with its file and line |
| A marker on a class is printed as one line, and no test is named beside it | A marker on a class is one line, and it names no test |
| It prints the tests of the result that the command shows, each with its outcome | The tests of the result that the command shows, each with its outcome |
| The second list also holds the tests that passed | A result keeps the name and the outcome of every test of its Check, and not only of the failed ones. |
| It says how old that result is | It is as old as that result, and the command says how old |
| It starts no launch, also for a Check that is owed | It prints two lists, and it starts no launch. |
| After an edit that adds a marked test, the first list shows the new marker and the second list is unchanged | The first list shows its marker at once. |
| With `--json` the document holds the fields `markers` and `tests` | The format `check-1` gains two fields for it, `markers` and `tests`. |
| `failing_tests` is still there and names what it named before | `failing_tests` stays as it is. |
| For a Check that is kept by an older pass while a newer fail is stored, the second list and `failing_tests` describe the kept pass | The result that the command shows is the one the state rests on: the kept result, or else the last one. |
| The document still names its format `check-1` | Adding a field does not raise the number of a format. |
| With `--json` an entry of `markers` holds `file`, the path from the root of the repository, and `line` | An entry of `markers` holds `file`, the path from the root of the repository, and `line`. |
| An entry of `tests` holds `name`, `outcome` and `duration_ms` | An entry of `tests` holds `name`, `outcome` and `duration_ms`, as a result keeps them. |
| For a Check with no result, `tests` is an empty list and the text says that no result exists yet | A Check with no result has an empty `tests`, and the text says that no result exists yet. |
| For a Check with no marker, `markers` is an empty list | A Check with no marker has an empty `markers`. |

## The site workflow

| Behavior | Quote from the design |
| --- | --- |
| Started by hand on a version tag, the site workflow publishes nothing and its job fails | a manual start of the site workflow from `main` |
| Called by the release workflow on a version tag, it publishes | a version tag, where the site follows the release |
| Started by hand on `main`, it publishes as before | a manual start of the site workflow from `main` |

## The release workflow

| Behavior | Quote from the design |
| --- | --- |
| One job of the release workflow pushes to NuGet and then publishes the draft | the workflow pushes to NuGet and then publishes the draft, in one job |
| Exactly one job of the release workflow names the environment `release` | GitHub asks for the approval once for each job, so one job holds everything that follows the approval. |
| That job may write to the contents of the repository and may ask for the identity for the registry, and no other job may do both | The job that follows the approval holds two rights at once: the identity for the registry and the right to write to the repository |

## Not in this pass

| Finding | Where it goes | Why |
| --- | --- | --- |
| Lint reads the file roots in `realization.md` and reports one that names no file | increment 3 | the design says so, and the written form of a file root is decided there |
| The brief of each review carries the six questions | increment 4 | the reviews are built there. Until then a person writes the brief |
| The suites of this repository run under `cargo-nextest`, CI runs in three jobs, and the tests of the launch limits are split | after this pass | they change how this repository is tested and no behavior of the product |
