# The skill for the form of an exploration: behaviors

This is the list of behaviors for a short pass between the second and the third increment of the core. `azimuth skills install` gains a second skill, `azimuth-exploration`, which says which files an exploration leaves, where they stand, and what a row of the map and a decision note hold. The skill for a change gains the acts that the design gives to a change: it moves a finished exploration, and it adds a row for what it leaves. A person reads this list before the build starts.

A row is of one of two kinds. A row of behavior is one observable fact, and a test shows it. A row of prose says what a text tells its reader; its section begins with the words "Rows of prose", and no test reads the words of the text for it. Each row is one observable fact. The second cell is copied exactly from `docs/design.md`. A plain test fails when a quoted sentence is no longer in that file. A behavior with no sentence to quote is not built.

Status of this list: not yet read by a person.

## `azimuth skills install`

| Behavior | Quote from the design |
| --- | --- |
| For either harness the command writes the text of the exploration skill to `.agents/skills/azimuth-exploration/SKILL.md` | Its text is always in `.agents/skills/azimuth-exploration/SKILL.md` |
| For either harness the command still writes the change skill to `.agents/skills/azimuth-change/SKILL.md` | **`azimuth skills install <harness>` installs two skills, one for a change and one for an exploration.** |
| For `claude` it adds a link at `.claude/skills/azimuth-exploration` | for `claude` the command adds a link at `.claude/skills/azimuth-exploration` |
| The link is relative, and the skill is still found through it after the clone is moved | The link is relative, so it survives a move of the clone. |
| The last line of the exploration skill file is its own digest line, `<!-- azimuth text-1:<hex> -->` over the file's text without that line | Each file carries its own digest line |
| Run again, the command replaces an exploration skill file that carries a correct digest line of another text | so an upgrade replaces each one only when it was not edited |
| Run again, it leaves an edited exploration skill file alone and names it, and still replaces an unedited change skill | so an upgrade replaces each one only when it was not edited |
| Run again, it leaves an edited change skill file alone and names it, and still replaces an unedited exploration skill | so an upgrade replaces each one only when it was not edited |
| In a repository that has only the change skill of the version before, the command writes the exploration skill and its link, and replaces the change skill | The command writes `.agents/skills/azimuth-exploration/SKILL.md` and its link, and replaces the change skill when nobody edited it. |
| `azimuth status` says that the process here differs from the default when only the exploration skill was edited | Status then says "the process here differs from the default" |
| A change that edits `.agents/skills/azimuth-exploration/SKILL.md` is of class D in `azimuth change diff` | **A change to an installed skill file is class D.** |
| A change that touches the link `.claude/skills/azimuth-exploration` is of class D in `azimuth change diff` | **A change to an installed skill file is class D.** |

## The command of the skill for a finished exploration

Step 1 of the change skill gives the git command for the day of a folder's arrival. A test takes the command for an exploration out of the installed text and runs it.

| Behavior | Quote from the design |
| --- | --- |
| For an exploration folder that arrived by a merge, the command prints the day of the merge on the target branch, and not the day of a commit on the change branch | the same way as a change folder, with the date of the folder's arrival in front of its name |
| For an exploration folder that arrived by a fast-forward, the command prints the day of the commit that first brought the folder | The date is then the day of the commit that first brought the folder, which can be earlier than the arrival. |
| For an exploration folder that is not on the target branch, the command prints nothing | the same way as a change folder, with the date of the folder's arrival in front of its name |

## The exploration skill: what it is for

Rows of prose: a person reads the text, and one test compares the whole text with what that person accepted.

| Behavior | Quote from the design |
| --- | --- |
| The description of the skill says that it is for the form of an exploration: which files it leaves, where they stand, and what a row of the map and a decision note hold | An exploration gets a skill of its own for its form: which files it leaves, where they stand, and what a row of the map and a decision note hold. |
| The text says nothing about how to explore | How to explore is not decided yet, and that skill says nothing about it. |
| It says that only work of several changes needs an exploration | only the largest needs an exploration |
| It says that a change belongs to an exploration only when a map lists its name, and the proposal does not have to say anything | A change belongs to an exploration when a map lists its name. The proposal does not have to say anything. |
| It says that an exploration can be born late, and its first row is then a change that is already finished | An exploration can be born late. Its first row is then a change that is already finished. |
| It says that explorations live in the repository that holds the model | Change folders and explorations live in the repository that holds the model |

## The exploration skill: the folder

Rows of prose: a person reads the text, and one test compares the whole text with what that person accepted.

| Behavior | Quote from the design |
| --- | --- |
| It says that an exploration is the folder `azimuth/explorations/<name>/` with `map.md` and `sources/` | `map.md` and `sources/`, with the decision notes |
| It says that the shared sources and the decision notes are first written in the exploration's `sources/`, and that the exploration is the pool from which a change copies what its row names | A shared source or a decision note is first written in the exploration's `sources/`. The exploration is the pool, and the row of the map names what to copy from it. |
| It says that a source that only one change needs is written in the folder of that change and not in the exploration | A source that only one change needs is written in the folder of that change and nowhere else. |
| It says that an exploration ends when every row is finished or deleted, and that the next change moves it to `azimuth/explorations/finished/` with the date of its arrival in front of its name | An exploration ends when every row is finished or deleted. |

## The exploration skill: sources

Rows of prose: a person reads the text, and one test compares the whole text with what that person accepted.

| Behavior | Quote from the design |
| --- | --- |
| It says that one snapshot is one file in `sources/`, with its address, version and date at the top | One snapshot is one file in `sources/`, with its address, version and date at the top |
| It says that a snapshot is never edited, and a newer version of the page comes in as a new snapshot | A snapshot is never edited. A newer version of the page comes in as a new snapshot |
| It says that a source written directly in the repository goes in the same folder | A source written directly in the repository goes in the same folder |
| It says that a person, or an agent with a connector, takes the snapshot, and that Azimuth downloads nothing | A person, or an agent with a connector, takes the snapshot. |

## The exploration skill: decision notes

Rows of prose: a person reads the text, and one test compares the whole text with what that person accepted.

| Behavior | Quote from the design |
| --- | --- |
| It says that a decision that holds for several changes is a decision note, and that neither the map, a proposal nor the model holds it | **A decision that holds for several changes is a source.** |
| It says that a note holds the date, who decided, and the words | A decision note is a file in the exploration's `sources/`, with the date, who decided, and the words |
| It says that notes stand together in `sources/decisions/`, one note to a file | Decision notes stand together in `sources/decisions/`, one note to a file |
| It says that a note's file name is the day of the decision and a short name, with an example | A note's file name is the day of the decision and a short name |
| It says that a note is never edited, and that a later decision is a new note that names the one it replaces | It is never edited. A later decision is a new note that names the one it replaces |
| It says that a note binds nobody by itself, and becomes binding when a person approves Scenarios that follow it | A note binds nobody by itself. It becomes binding when a person approves Scenarios that follow it |

## The exploration skill: the map

Rows of prose: a person reads the text, and one test compares the whole text with what that person accepted.

| Behavior | Quote from the design |
| --- | --- |
| It says that the map is one table with the columns "Change", "Takes from the sources" and "After", and that a row names what its change takes from the sources and which rows must be finished first | it names what the change takes from the sources and which rows must be finished first |
| It says that a row carries the name its change will get, a plain change name, and that only work under another model gets a prefixed row | A row of the map is a plain change name |
| It says that the state of a row is never typed | Nobody types it |
| It says how the agent reads the state of a row: finished when the folder of its change stands on the target branch, in `changes/` or in `changes/finished/` | A row is finished when the folder of its change stands there, in `changes/` or in `changes/finished/`. |
| It says that "After" names the rows that must be finished first, and that it is the only ordering | "After" names rows that must be finished first |
| It says that the map is edited freely, in ordinary commits | The map is edited freely, in ordinary commits |
| It says that a row that is split becomes two rows in the table | the row becomes two rows |
| It says that an exploration is never approved, and a person approves each proposal when it is written | An exploration is never approved |
| It says that a row may exist with no change behind it for as long as needed | A row may exist with no change behind it for as long as needed |
| It says that a row cites a decision note by its short name alone | A row of the map cites the note by the short name alone |
| It says that whoever writes a decision note adds its short name to every row it bears on, in the same commit | Whoever writes a decision note adds its short name to every row it bears on, in the same commit |
| It says that after its names a row may say in one line which part of a source its change takes, or what the work is when no source says it | After its names, a row may say in one line which part of a source its change takes, or what the work is when no source says it. |
| It says that a decision is never written only in a row | A decision is never written only in a row |
| It says that a row is deleted only in a commit that adds a decision note saying why | A row is deleted only in a commit that adds a decision note saying why |

## The exploration skill: leftovers

Rows of prose: a person reads the text, and one test compares the whole text with what that person accepted.

| Behavior | Quote from the design |
| --- | --- |
| It says that every leftover of a change is a row in the map, and that the model never holds a promise | **Every leftover of a change is a row in the map. The model never holds a promise.** |
| It says that work not started, a release toggle to remove, and the contract half of an expand and contract migration are each a row | The contract half of an expand and contract migration |
| It says that a Scenario that is wanted and not built goes into the sources and a row, and not into the model | the sources and a row. Not the model |
| It says that evidence that is weak or owed gets no row, because status already shows it | nowhere. Status already shows it |

## The change skill: a finished exploration

Rows of prose: a person reads the text, and one test compares the whole text with what that person accepted.

| Behavior | Quote from the design |
| --- | --- |
| Step 1 tells the agent, before it writes the proposal, to move every exploration whose rows are all finished or deleted into `azimuth/explorations/finished/<date>-<name>` | The next change then moves its folder to `explorations/finished/` |
| It says how the agent reads that a row is finished: the folder of its change stands on the target branch, in `azimuth/changes/` or in `azimuth/changes/finished/` | A row is finished when the folder of its change stands there, in `changes/` or in `changes/finished/`. |
| It gives the git command for the day on which the exploration folder arrived on the target branch, and tells the agent not to type the date | the same way as a change folder, with the date of the folder's arrival in front of its name |
| Where it reads a row of the map, step 1 names the skill `azimuth-exploration` as the text that says what a row holds | The skill for a change reads that form, so the form cannot stay a habit. |

## The change skill: what a change leaves

Rows of prose: a person reads the text, and one test compares the whole text with what that person accepted.

| Behavior | Quote from the design |
| --- | --- |
| Before the stop of the hand over, the agent asks what the change leaves undone, and adds a row to the map for each leftover | A change that leaves something adds its row before it merges. |
| It says that a change with no exploration that leaves something creates one, with itself as the first row | A change with no exploration that leaves something creates one at that moment, with itself as the first row. |
| It says that the agent proposes a name for the work that the map holds, not for the change, and names the new exploration at the hand over | The agent proposes a name for the work that the map holds, not for the change, and names the new exploration at the hand over. |
| It says that the person may rename the exploration before the merge | The person may rename it before the merge. |

## Not in this pass

| Finding | Where it goes | Why |
| --- | --- | --- |
| A skill file that a person deleted is written again by the next install, and `azimuth status` says `process: default` while it is missing | a later pass, after a sentence of the design | the design says nothing about a deleted skill file, and this pass does not change how the install treats one |
| The method of an exploration: how the person and the agent explore | after the research and the talk that follow this pass | the design says that how to explore is not decided |
| A command that computes the state of a row | a later increment | until then the agent reads the state from the target branch |
