# The skill for exploring an initiative: behaviors

This is the list of behaviors for a short pass after the skill for the form of an initiative. `azimuth skills install` gains a third skill, `azimuth-exploration`, which says how a person and an agent work an initiative out: the request first, reading before asking, one question at a time, the cut into rows, and a fresh reader. The skill for a change gains the acts that the design gives to a change around an exploration: no proposal stop for a change of an initiative, the question at the proposal stop of a change with no initiative, the test for what a change finds, and an exploration opened again for one decision. The skill for the form gains the part of the map for what is not cut yet. A person reads this list before the build starts.

A row is of one of two kinds. A row of behavior is one observable fact, and a test shows it. A row of prose says what a text tells its reader; its section begins with the words "Rows of prose", and no test reads the words of the text for it. Each row is one observable fact. The second cell is copied exactly from `docs/design.md`. A plain test fails when a quoted sentence is no longer in that file. A behavior with no sentence to quote is not built.

Status of this list: read and accepted by a person on 2026-11-21, in one sitting. Later that day, after an independent reading of the texts, the person agreed to two added rows of prose: a line under "Not cut yet" keeps an initiative open, in the initiative skill and in step 1 of the change skill. The person also agreed to a row on where a change with no initiative writes a decision that holds for several changes. And to a row on a change of an initiative whose row turns out to be too large. And to a row on deleting the change folder when the person chooses an initiative. After a second reading, the person agreed that this row first moves the change folder's sources into the initiative. The person also agreed to a row on a change that joins an initiative after its proposal stop. And to a row on a fresh reader for the new rows of a row that is cut again. After a third reading, the person agreed to a row on choosing one change at the proposal stop. And to a row on the first of the new rows of a row cut again. And to a row on what a change that waits or ends keeps. After a fourth reading, the person agreed that an ended change merges nothing but what it changed in the initiative. Then the person agreed that a change whose row is cut again always ends: the two rows on its first new row and on waiting or ending were deleted, and one row says that it ends. After a fifth reading, the person agreed that its branch merges without the change folder, so the moves of step 1 merge too.

## `azimuth skills install`

| Behavior | Quote from the design |
| --- | --- |
| For either harness the command writes the text of the exploration skill to `.agents/skills/azimuth-exploration/SKILL.md` | The skill for exploring is `azimuth-exploration`, in `.agents/skills/azimuth-exploration/SKILL.md`, with the same kind of link for `claude`. |
| For either harness the command still writes the change skill and the initiative skill | **`azimuth skills install <harness>` installs three skills: one for a change, one for the form of an initiative, and one for exploring an initiative.** |
| For `claude` it adds a link at `.claude/skills/azimuth-exploration` | The skill for exploring is `azimuth-exploration`, in `.agents/skills/azimuth-exploration/SKILL.md`, with the same kind of link for `claude`. |
| The link is relative, and the skill is still found through it after the clone is moved | The link is relative, so it survives a move of the clone. |
| The last line of the exploration skill file is its own digest line, `<!-- azimuth text-1:<hex> -->` over the file's text without that line | Each file carries its own digest line |
| Run again, the command replaces an exploration skill file that carries a correct digest line of another text | so an upgrade replaces each one only when it was not edited |
| Run again, it leaves an edited exploration skill file alone and names it, and still replaces an unedited change skill and an unedited initiative skill | so an upgrade replaces each one only when it was not edited |
| Run again, it leaves an edited change skill file alone and names it, and still replaces an unedited exploration skill | so an upgrade replaces each one only when it was not edited |
| In a repository that has only the change skill and the initiative skill of the version before, the command writes the exploration skill and its link, and replaces the other two | **`azimuth skills install <harness>` installs three skills: one for a change, one for the form of an initiative, and one for exploring an initiative.** |
| `azimuth status` says that the process here differs from the default when only the exploration skill was edited | Status then says "the process here differs from the default" |
| A change that edits `.agents/skills/azimuth-exploration/SKILL.md` is of class D in `azimuth change diff` | **A change to an installed skill file is class D.** |
| A change that touches the link `.claude/skills/azimuth-exploration` is of class D in `azimuth change diff` | **A change to an installed skill file is class D.** |

## The exploration skill: what it is for

Rows of prose: a person reads the text, and one test compares the whole text with what that person accepted.

| Behavior | Quote from the design |
| --- | --- |
| The description of the skill says that it is for exploring an initiative: how a person and an agent work it out | one for exploring an initiative |
| It names the skill `azimuth-initiative` for which files an initiative leaves and what a row and a note hold, and does not restate that form | A rule written in two places goes stale in one of them. |
| It says that an exploration starts when a person asks for one, or when the person chooses it at the proposal stop of a change with no initiative | **An exploration starts when a person asks for one, or when the person chooses it at the proposal stop of a change with no initiative.** |

## The exploration skill: the request and reading first

Rows of prose: a person reads the text, and one test compares the whole text with what that person accepted.

| Behavior | Quote from the design |
| --- | --- |
| It says that the first source is the request in the person's own words, written as a source in the repository before the agent asks anything | **The first source is the request in the person's own words.** |
| It says that the agent takes a snapshot of every page that the request names | takes a snapshot of every page that the request names |
| It says that later only what stands in a source written by the person, or in a note that the person approved, counts as the person's | Later, only what stands in a source written by the person, or in a note that the person approved, counts as the person's. |
| It says that before its first question the agent reads the sources and the code that the request touches, and asks the person only what neither answers | **Before its first question, the agent reads the sources and the code that the request touches. It asks the person only what neither answers.** |

## The exploration skill: the questions

Rows of prose: a person reads the text, and one test compares the whole text with what that person accepted.

| Behavior | Quote from the design |
| --- | --- |
| It carries the six questions of the design, each with what it catches | The skill for an exploration carries the list. |
| It says that a question with an answer becomes a decision note or a row, and that a question that does not apply leaves nothing | **A question with an answer becomes a decision note or a row. A question that does not apply leaves nothing.** |
| It says that the agent asks one question at a time | **The agent asks one question at a time.** |
| It says that when the person's own view is not known yet, the question is open and carries no example answer, and options come after the answer | **When the person's own view is not known yet, the question is open and carries no example answer.** Options come after the answer, not before it. |

## The exploration skill: decisions

Rows of prose: a person reads the text, and one test compares the whole text with what that person accepted.

| Behavior | Quote from the design |
| --- | --- |
| It says that for a decision the agent lays out three real approaches and recommends one, and that when only two are real it shows two and says why there is no third | **For a decision, the agent lays out three real approaches and recommends one. When only two are real, it shows two and says why there is no third.** |
| It says that the agent shows the exact words of the note, its cost and one example, and writes the note only after the person agrees to those words | It shows the exact words of the note, its cost and one example, and writes the note only after the person agrees to those words. |
| It says that before it shows a decision the agent tries it on cases that it invents: the edge of a rule, two rules at once, a thing that changes while it is in use | **Before it shows a decision, the agent tries it on cases that it invents: the edge of a rule, two rules at once, a thing that changes while it is in use.** |
| It says that a case that the decision does not answer becomes a question | A case that the decision does not answer becomes a question. |
| It says that a note says whose words it holds: the person's own words are quoted, and the agent's words that the person agreed to are marked as the agent's | **A note says whose words it holds.** The person's own words are quoted. The agent's words that the person agreed to are marked as the agent's. |

## The exploration skill: the cut

Rows of prose: a person reads the text, and one test compares the whole text with what that person accepted.

| Behavior | Quote from the design |
| --- | --- |
| It says that the agent cuts the work into rows, each one change: one outcome, and a proposal that will fit on one page | **The agent cuts the work into rows, each one change: one outcome, and a proposal that will fit on one page.** |
| It says that a row is cut by what somebody can do after it, not by layer | **A row is cut by what somebody can do after it, not by layer.** |
| It says that a row for one layer is allowed when it can be deployed alone and is useful alone, and that its decision note then says what nobody can do between it and the row that completes it | A row for one layer is allowed when it can be deployed alone and is useful alone. Its decision note then says what nobody can do between it and the row that completes it. |
| It says that what is not understood yet stays uncut, named in one line in the part "Not cut yet", and is cut only when a question about it can be stated precisely | the agent cuts it only when a question about it can be stated precisely |
| It says that the cut names its order in the column "After", and that a row waits only for the rows that it really needs | **The cut names its order in the map's column "After".** A row waits only for the rows that it really needs. |
| It says that before the first row starts the person reads the cut: whether each row is one outcome, whether anything is missing or doubled, and whether "After" is right | **Before the first row starts, the person reads the cut: whether each row is one outcome, whether anything is missing or doubled, and whether "After" is right.** |

## The exploration skill: the fresh reader

Rows of prose: a person reads the text, and one test compares the whole text with what that person accepted.

| Behavior | Quote from the design |
| --- | --- |
| It says that after the cut, and before the person reads it, a session that took no part in the talk reads each row as its change will: the row and what it names, not the whole folder | **After the cut, and before the person reads it, a session that took no part in the talk reads each row as its change will: the row and what it names, not the whole folder.** |
| It says that the reader reports only where an agent writing that proposal would have to guess, and where a wrong guess would matter | It reports only where an agent writing that proposal would have to guess, and where a wrong guess would matter. |
| It says that the person reads the findings together with the cut, and that a finding is repaired in the map or in a new decision note, never in the talk alone | **The person reads the findings together with the cut. A finding is repaired in the map or in a new decision note, never in the talk alone.** |

## The change skill: the proposal

Rows of prose: a person reads the text, and one test compares the whole text with what that person accepted.

| Behavior | Quote from the design |
| --- | --- |
| It says that a change of an initiative has no proposal stop: the agent writes the proposal from the row and goes on to the Intent step | Its row was agreed in the exploration, so the agent writes the proposal from the row and goes on to the Intent step. |
| It says that the person then reads the proposal at the Intent stop, beside the Scenarios | The person reads the proposal at the Intent stop, beside the Scenarios. |
| It says that a change with no initiative keeps the proposal stop | A change with no initiative keeps the stop |
| It says that a change that joins an initiative after its proposal stop keeps the approval it got there | A change that joins an initiative after its proposal stop keeps the approval it got there. |
| It says that a change of an initiative whose row turns out to have two outcomes, or a proposal that will not fit on one page, stops before the Intent step and proposes how to cut the row again | **A change of an initiative whose row turns out to have two outcomes, or a proposal that will not fit on one page, stops before the Intent step.** |
| It says that when the person agrees to the new cut, the change ends: the agent deletes its folder, and its branch merges without it, so the new rows wait for changes of their own | When the person agrees, the change ends: the agent deletes its folder, and its branch merges without it, so the new rows wait in the map for changes of their own. |
| It says that before the person agrees to the new cut of a row, a fresh reader reads the new rows, as after any cut | Before the person agrees, a fresh reader reads the new rows, as after any cut. |
| It says that when a proposal would not fit on one page or has two outcomes, the agent says so at the proposal stop and asks: one change, or an initiative, and does not stop earlier for it | When a proposal would not fit on one page or has two outcomes, the agent says so at that stop and asks: one change, or an initiative. It does not stop earlier for it. |
| It says that when the person chooses one change at the proposal stop, that answer approves the proposal as it is | When the person chooses one change, that answer approves the proposal as it is. |
| It says that when the person chooses an initiative, the agent goes on with the skill `azimuth-exploration` | or when the person chooses it at the proposal stop of a change with no initiative |
| It says that when the person chooses an initiative, the agent moves the sources of the change folder into the initiative's `sources/`, deletes the folder with its proposal, and the initiative starts from the request | When the person chooses an initiative, the agent moves the sources of the change folder into the initiative's `sources/`, deletes the folder with its proposal, and the initiative starts from the request. |

## The change skill: what a change finds

Rows of prose: a person reads the text, and one test compares the whole text with what that person accepted.

| Behavior | Quote from the design |
| --- | --- |
| It says that what a change finds is repaired inside it or becomes a row, by one test: does it make a Scenario of this change untrue? | **What a change finds is repaired inside it or becomes a row, by one test: does it make a Scenario of this change untrue?** |
| It says that a fault that makes an approved Scenario of this change untrue is repaired inside the change | A fault that makes an approved Scenario of this change untrue |
| It says that a fault older than the change, outside what the change promises, becomes a row in the map | A fault older than the change, outside what the change promises |
| It says that a repair that no test can reach, or half of a wider repair, is a limit named at the hand over and a row | A repair that no test can reach, or half of a wider repair |
| It says that the person may choose otherwise at any stop, and the agent follows the person | The person may choose otherwise at any stop, and the agent follows the person. |

## The change skill: a decision for several changes

Rows of prose: a person reads the text, and one test compares the whole text with what that person accepted.

| Behavior | Quote from the design |
| --- | --- |
| It says that when a change finds that a decision holds for several changes, the exploration opens again for that one decision, and the agent asks it as the skill `azimuth-exploration` says | **An exploration opens again for one decision when a change finds that a decision holds for several changes.** |
| It says that the agent writes the note in the initiative's `sources/decisions/` and adds its name to every row that it bears on | It writes the note in the initiative's `sources/decisions/` and adds its name to every row that it bears on. |
| It says that a row that the decision makes wrong is changed or deleted in the same commit | A row that the decision makes wrong is changed or deleted in the same commit. |
| It says that a change with no initiative writes the note into an open initiative that the work belongs to, and creates one with itself as the first row only when none fits | A change with no initiative writes the note into an open initiative that the work belongs to. Only when none fits does it create one, with itself as the first row, as it does for its leftovers. |
| It says that a finished row is not opened again, and what the decision asks of it becomes a new row | **A finished row is not opened again.** What the decision asks of it becomes a new row. |

## The change skill: a finished initiative

Rows of prose: a person reads the text, and one test compares the whole text with what that person accepted.

| Behavior | Quote from the design |
| --- | --- |
| Step 1 moves an initiative to `azimuth/initiatives/finished/` only when nothing stands under "Not cut yet" in its map | A line under "Not cut yet" keeps it open, until the line is cut into rows or dropped with a decision note that says why. |

## The initiative skill: what is not cut yet

Rows of prose: a person reads the text, and one test compares the whole text with what that person accepted.

| Behavior | Quote from the design |
| --- | --- |
| It says that the map has a part "Not cut yet", which names in one line each piece of the work that is not understood yet | The map names it in one line in a part "Not cut yet" |
| It says that a line under "Not cut yet" keeps the initiative open, until the line is cut into rows or dropped with a decision note that says why | A line under "Not cut yet" keeps it open, until the line is cut into rows or dropped with a decision note that says why. |

## Not in this pass

| Finding | Where it goes | Why |
| --- | --- | --- |
| The change skill pushes its branch at the start, so that a row shows as taken | after a decision of the person | it is not decided, and a push is an act outside the machine |
| Scheduling on the server, the commands for whole changes, and the coordinator | the server rung | until then the order lives in "After" and a developer picks the next ready row |
| Whether an agent that reads the skill really asks one question at a time | not tested | a language model inside a test needs a deliberate decision of a person, and none was made |
| Examples from drim-dev that this pass added to the design | a separate step, after a decision of the person | new examples use an invented product, and replacing the old ones is not decided |
