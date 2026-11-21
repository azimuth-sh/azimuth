# After the first use of increment 2: behaviors

This is the list of behaviors for a short pass between the second and the third increment of the core. The first real change that was made with increment 2 added rules to `docs/design.md` that the shipped skill does not say yet, and it asked for one more list in `azimuth check show`. A person reads this list before the build starts.

A row is of one of two kinds. A row of behavior is one observable fact, and a test shows it. A row of prose says what a text tells its reader; its section begins with the words "Rows of prose", and no test reads the words of the text for it. Each row is one observable fact. The second cell is copied exactly from `docs/design.md`. A plain test fails when a quoted sentence is no longer in that file. A behavior with no sentence to quote is not built.

Status of this list: read and accepted by a person on 2026-11-20, in eight sittings. The last one changed one row and added two after an independent reading of the skill text against this list.

## The skill: finished folders

Rows of prose: a person reads the text, and one test compares the whole text with what that person accepted.

Step 1 of the shipped skill already tells the agent to move every change folder that exists on the target branch into `azimuth/changes/finished/`.

| Behavior | Quote from the design |
| --- | --- |
| Step 1 names the moved folder `<date>-<name>` | The moved folder is named `<date>-<name>`. |
| It says which day the date is: the day on which the folder arrived on the target branch | The date is the day on which the folder arrived on the target branch. |
| It gives one git command that prints that day in UTC, and tells the agent not to type the date | The process reads it from git, in UTC |
| Step 1 tells the agent to read the day in a clone that has the whole history of the target branch | The process reads the day in a clone that has the whole history of the target branch. |
| Step 1 says where the target branch is named: in `azimuth/project.toml` | The target branch is named in the project file |
| Step 1 says that the change gets a name that no change used before | A change name is never used twice. |

## The skill: the proposal

Rows of prose: a person reads the text, and one test compares the whole text with what that person accepted.

| Behavior | Quote from the design |
| --- | --- |
| Step 1 says what the Outcome holds: what somebody can do or observe after the change that they cannot now, in their words | what somebody can do or observe after the change that they cannot now, in their words |
| It says what the Sources hold: the sources and decision notes that the proposal relies on, and which part of each | the sources and decision notes it relies on, and which part of each |
| It says what the Boundary holds: what the change leaves out, each item with the name of its map row | what it leaves out, each item with the name of its map row |
| It says what the Decisions hold: what this change settles, each with the alternative it rejects, one line each | what this change settles, each with the alternative it rejects, one line each |
| It says what the Approver is: who is expected to approve the Scenarios | who is expected to approve the Scenarios |
| It says that a proposal holds no Scenarios, no plan and no completion conditions | It holds no Scenarios, no plan and no completion conditions. |
| It says that a proposal that does not fit on one page, or has two outcomes, is several changes | A proposal that does not fit on one page, or has two outcomes, is several changes. |
| It says that a proposal is not edited after approval | A proposal is not edited after approval. |
| It says that a change of an initiative is written from its row of the map, and takes its name from the row | A change of an initiative is written from its row of the map. The row gives the change its name |
| It says that a change with no initiative is written from the request of the person, and from `sources/` in its own folder when it has one | A change with no initiative is written from the request of the person, and from `sources/` in its own folder when it has one. |
| It says that the agent asks the person for what it cannot find there, and does not invent it | What the agent cannot find there, it asks the person and does not invent. |
| It says that a change of an initiative copies the files that its row names into `azimuth/changes/<name>/sources/` | It copies those files into the `sources/` of the change. |
| It says that a copy is the whole file, unchanged, under the same path inside `sources/` | A copy is the whole file, unchanged, under the same path inside `sources/`. |
| It says that the Sources part names each source by its path from the change folder | the proposal names each source by its path from that folder |
| It says that a source that only this change needs is written in the folder of this change and nowhere else | A source that only one change needs is written in the folder of that change and nowhere else. |
| It says that a copy is not touched when a later source replaces it | A copy is not touched when a later source replaces it. |
| The stop of step 1 and the stop of step 2 each end with the approval of the person, and after a request for a change the agent makes the change and stops again | a person approves each proposal, at its own stop or, for a change of an initiative, at the Intent stop, and that is the only decision |

## The skill: Scenarios

Rows of prose: a person reads the text, and one test compares the whole text with what that person accepted.

| Behavior | Quote from the design |
| --- | --- |
| Step 2 says that a Requirement and its Scenarios speak about the system as somebody outside it meets it | A Requirement and its Scenarios speak about the system as somebody outside it meets it: a learner, an admin, an operator, another system |

## The skill: the parts of a Check

Rows of prose: a person reads the text, and one test compares the whole text with what that person accepted.

| Behavior | Quote from the design |
| --- | --- |
| Step 2 says that a Check may be made of a test through a door of the system and of tests of one part in isolation | A Check may be made of parts: a test through a door of the system, and tests of one part in isolation. |
| It says that every part carries the marker of the Check | All of them carry the marker of the Check |
| It says that a Check is never made only of tests of one part in isolation | A Check is never made only of tests of one part in isolation. |
| It says that a test through a door is needed once for each place where the part is wired | It is needed once for each place where the part is wired, for example once for each operation that has a validator, and not once for a Requirement. |
| It gives the order of the claim of such a Check: the door, what each part shows, and last what is not shown | Its order is fixed: through which door the test looks, what each part shows, and last what is not shown. |
| It says that a journey, one test through several real operations, may be a part of several Checks and carries the marker of each Check that relies on it | It carries the marker of every Check that relies on it. |
| It says that the claim of each such Check names the stretch of the journey that it relies on | The claim of each such Check names the stretch of the journey that it relies on |
| It says that a Check also has a test of one operation alone, where that operation can be tested alone | Where one operation can be tested alone, the Check also has a test of that operation alone |
| It says that when the change removes a Check, the marker leaves, and its test leaves only when the test carries no other marker | Its test leaves with it only when the test carries no other marker. |
| It says what a Check is and where it stands: one claim about its Scenario, under that Scenario in `verification.md` | A Check states one claim about a Scenario |
| It says that a Check belongs to one suite, and that evidence from another suite is another Check under the Scenario | Evidence from two suites means two Checks under one Scenario. |
| Step 3 says what a marker is and where it stands: it enrolls a test in a Check, on a test or on a class, in the form that the suite declares in `azimuth/project.toml` | A marker enrolls a test in a Check. |

## The skill: the plan

Rows of prose: a person reads the text, and one test compares the whole text with what that person accepted.

| Behavior | Quote from the design |
| --- | --- |
| Step 2 says that the change needs a plan when a Requirement that it adds or changes declares `Realization:` | A change needs a plan when a Requirement that it adds or changes declares `Realization:` |
| It says that a change with no such Requirement needs no plan | A change with no such Requirement needs no plan |
| It says that the plan covers the whole change, also what is built for a Requirement that declares no `Realization:` | A plan covers the whole change, also what the change builds for a Requirement that declares no `Realization:` |
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

Rows of prose: a person reads the text, and one test compares the whole text with what that person accepted.

| Behavior | Quote from the design |
| --- | --- |
| Step 3 tells the agent to move a proposed decision into `realization.md` only when the finished code follows it | It moves to `realization.md` only if the finished code follows it. |
| Step 3 puts the tests through a door before the code, and the tests of one part in isolation after it, written from the Scenario and not from the code | A test of one part in isolation needs the part and is written after it, from the Scenario and not from the code. |
| Step 3 puts the writing of `realization.md` before the last run | `realization.md` is written before the last run of the Build. |
| Step 3 says where `realization.md` stands: beside `specification.md`, in the folder of the module | specification.md, realization.md and verification.md |
| Step 3 says what `realization.md` holds: the heading of the Requirement again, its roots, and its decisions with their reasons, where a root marks where the realization starts | A root marks where the realization of a Requirement starts. |
| Step 3 ends when `azimuth run` succeeds and `azimuth status` names nothing that a run could settle | runs `azimuth run` until it succeeds and `azimuth status` names nothing that a run could settle |
| It says what may be left: a review, which this version does not record, a Check that runs only in CI, and a Check that cannot run here | a review, which this version does not record, a Check that runs only in CI, or a Check that cannot run here |
| It says that a Check that a run here could settle is work of the Build | A Check that a run here could settle is work of the Build. |
| Step 4 tells the agent to name every proposed decision that the finished code does not follow, and to say why | At the hand over the agent names every proposed decision that the finished code does not follow, and says why |
| Step 4 tells the agent to name what `azimuth status` still names | The agent names what is left at the hand over. |
| Step 4 says that a declared review is still owed and is done outside Azimuth | Until increment 4 the review is done outside Azimuth, and status shows it as owed. |
| Step 4 says what follows a commit lane that fails: after status 1 the agent repairs the code or the tests, after 2 the model, and it runs the lane again | after 1 an agent fixes the code or the tests, and after 2 it fixes the model, the command line or the installation |
| Step 4 says that when the commit lane fails because a Check cannot run here, the agent names that Check, says that the commit lane of CI decides, and repairs nothing for it | The agent names that Check and says that the commit lane of CI decides. |
| Step 4 tells the agent to name a proposed decision that served a Requirement with no `Realization:` as archived with the plan | A decision for such a Requirement has no `realization.md` to move to, and it is archived with the plan |

## The command of the skill

Step 1 of the skill gives one git command. A test takes it out of the installed text and runs it.

| Behavior | Quote from the design |
| --- | --- |
| For a folder that arrived by a merge, the command of the skill prints the day of the merge on the target branch, and not the day of the commit on the change branch | The date is the day on which the folder arrived on the target branch. |
| Run on two branches that start from the same target branch, the command prints the same day | both made the same rename, because both read the same date from git and nobody types it |
| For a folder that arrived by a fast-forward, the command prints the day of the commit that first brought the folder | The date is then the day of the commit that first brought the folder, which can be earlier than the arrival. |

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
| For a Check that is kept by an older pass while a newer fail is stored, the second list and `failing_tests` describe the kept pass | The result that the command shows is the one the state rests on: the kept pass, or the fail on these inputs, or, when the state rests on neither, the newest result. |
| The text says which result it shows: the kept pass, the fail on these inputs, or the newest result | The text says which of the three it is. |
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
| Lint reports a path in a proposal that names no file, and a copy of a source that differs from its origin | increment 3 | the design says so, and lint learns to read files of a change there |
| A second run of the site job can fail, because the upload of the first run is still there | a later pass | it is older than this pass. Until then a person starts the workflow again and does not run the job again |
| The sentence about the profile that calls a journey Check the marked exception to the rule of one endpoint | increment 6 | a journey is a test that several Checks share. The profile is built there, and nothing reads the sentence before |
| The brief of each review carries the six questions | increment 4 | the reviews are built there. Until then a person writes the brief |
| The suites of this repository run under `cargo-nextest`, CI runs in three jobs, and the tests of the launch limits are split | after this pass | they change how this repository is tested and no behavior of the product |
