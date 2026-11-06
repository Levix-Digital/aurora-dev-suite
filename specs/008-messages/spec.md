# Feature Specification: Messages between projects

**Feature Branch**: `feat/008-messages`

**Created**: 2026-11-06

**Status**: Approved (gate 3, 2026-11-06)

**Input**: User description: "Talk to the other projects from Aurora: a Messages view that shows first the conversations whose closing record waits for the user (conclusion, justification, positions, decision needed, after approval) with a decision or a request for changes written right there, a field to step into any conversation at any time, the conversations with their cycles and rounds, and sending and replying; through dcp-integrations, the Rust integrations provider shipped with Aurora, with the commands, header and rules of the provisional inbox tool 0.3.0 (ADR-0009, ADR-0010). Backlog items FEAT-029, FEAT-031, FEAT-032, FEAT-033 and FEAT-034, release v0.3.0 (Levi, 2026-11-06)."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Decide on what the projects agreed (Priority: P1)

A Levix developer opens Aurora and sees, at the top of the Messages view, every conversation between their projects that has closed and waits for them. Each one shows the report the projects wrote for them: who discussed it and in how many rounds, the subject, whether it ended in agreement or impasse, the conclusion, the justification, each project's position (impasse), the decision needed with the recommended option, and what each project will do after approval. They pick an option and the decision goes to every project in the conversation.

**Why this priority**: it is the one moment the model needs the user (ADR-0009); today it happens by reading Markdown in a terminal.

**Independent Test**: In the preview, open Messages, check that the sample closing record of `aurora-dev-suite` and `dev-cycle-protocol` is first with its six report fields, pick the recommended option, and check that the conversation leaves the waiting list and shows the decision.

**Acceptance Scenarios**:

1. **Given** registered projects that use messages, **When** Messages opens, **Then** Aurora lists, first and apart, the closing records that wait for the user, read through `dcp-integrations check --json` of each project that opened them; only the opening project's record is listed, so a conversation appears once.
2. **Given** a closing record, **When** it is shown, **Then** its fields come from `dcp-integrations record <thread> --json`: projects and rounds, cycle, subject, outcome, conclusion, justification, positions (impasse only), open points (when not "none"), decision needed and after approval, and the user's earlier requests in this conversation.
3. **Given** a closing record, **When** the user picks an option (or writes a choice) and confirms, **Then** Aurora runs `dcp-integrations decide <thread> --choice <text> --json` in the opening project and the record leaves the waiting list.
4. **Given** nothing waits, **When** Messages opens, **Then** the waiting section says "Nothing waits for your decision".

---

### User Story 2 - Ask for changes or step into a conversation (Priority: P1)

The developer is not satisfied with a conclusion, or wants to steer a conversation that is still running. They write their request in the conversation; it goes to every project in it, and the projects start a new cycle of up to three rounds that ends in a new closing record for them. The cycle repeats until they decide.

**Why this priority**: asked for by Levi on 2026-11-06; without it the only answers are yes or no.

**Independent Test**: In the preview, on the sample closing record press "Ask for changes", write a request, send it, and check that the conversation shows cycle 2 waiting for the projects; on a running sample conversation write a request in its field and check the same.

**Acceptance Scenarios**:

1. **Given** a closing record, **When** the user presses "Ask for changes", writes a request and sends it, **Then** Aurora runs `dcp-integrations decide <thread> --revise <text> --json`, the record leaves the waiting list, and the conversation shows the new cycle and the request.
2. **Given** any conversation, open or closed, **When** the user writes in its field and sends, **Then** Aurora runs `dcp-integrations guide <thread> --text <text> --json` in the project the conversation is listed under, and the conversation shows the new cycle and the request.
3. **Given** an empty request, **When** the user tries to send it, **Then** sending stays disabled and nothing runs.

---

### User Story 3 - Follow the conversations (Priority: P2)

Below what waits, the developer sees every conversation of every registered project: the subject, the projects, the kind and status of its last message, its cycle and round, and whether a message they sent is still unread after two days; opening one shows its messages in order, each with its sections as fields.

**Why this priority**: the user needs context to decide and to know when to step in; it also replaces reading the inbox branches by hand.

**Independent Test**: In the preview, open Messages, filter by `gedo`, open the sample conversation and check its three messages in order with their sections.

**Acceptance Scenarios**:

1. **Given** registered projects, **When** Messages opens, **Then** Aurora reads each project's messages through `dcp-integrations check --json` and `list --json`, at most four projects at once, and lists conversations newest first, grouped by thread.
2. **Given** a conversation, **When** it is opened, **Then** its messages are shown oldest first with sender, receiver, kind, cycle, round, date and each section (`show --json` returns them as fields); opening a message marks it seen through the provider.
3. **Given** a filter by project or "waiting for a project's answer", **When** it is chosen, **Then** the list shows only the matching conversations.
4. **Given** a sent message still unread after 48 hours, **When** the list is shown, **Then** the conversation is marked "no answer yet".

---

### User Story 4 - Start a conversation or answer (Priority: P2)

The developer starts a conversation from one project to another (a question or a proposal) or answers a message, from the skeleton of its kind; the form refuses a message whose required sections are empty, naming them.

**Why this priority**: lets the user talk to the projects without a terminal; most messages are still written by work sessions.

**Independent Test**: In the preview, press "New conversation", pick `aurora-dev-suite` to `gedo`, kind proposal, leave Proposal empty, check the refusal naming it, fill it and send.

**Acceptance Scenarios**:

1. **Given** "New conversation", **When** the user picks the sending project, the receiving project (from the sender's counterparts) and the kind, **Then** the form shows the sections of `dcp-integrations template <kind> --json`, with the required ones marked.
2. **Given** a filled form, **When** the user sends, **Then** Aurora runs `dcp-integrations send --to <slug> --kind <kind> --subject <subject> --body-file <file> --json` in the sender's folder and shows the new conversation.
3. **Given** a message, **When** the user answers with a kind from those the provider allows, **Then** Aurora runs `dcp-integrations reply <id> --kind <kind> --body-file <file> --json`; after three rounds of the current cycle only agreement or impasse are offered.
4. **Given** a refusal from the provider (empty sections, round limit, unknown project), **When** it answers, **Then** the message is shown next to the form and nothing is sent.

---

### User Story 5 - Get news without asking (Priority: P2)

While Aurora is open, it checks every registered project's inbox at a set interval (default five minutes) with the provider's cheap poll, and shows a count on the Messages entry of the navigation when something new arrives or a record waits.

**Why this priority**: the reason to have messages in Aurora at all: Levi keeps Aurora open.

**Independent Test**: In the preview, wait for the simulated poll and check the count on "Messages"; in the desktop with the provider, check that a poll with no change starts no fetch (the provider's log).

**Acceptance Scenarios**:

1. **Given** Aurora open, **When** the interval passes, **Then** Aurora runs `dcp-integrations poll --json` for each project that uses messages, at most four at once, and reads that project again only when the poll reports news.
2. **Given** news, **When** the poll answers, **Then** the navigation shows the number of conversations with news or waiting for the user.
3. **Given** the interval set to "off" in Settings, **When** Aurora is open, **Then** no poll runs and Messages reads only when opened or refreshed.

---

### User Story 6 - Use the provider from a terminal (Priority: P1)

A developer or a work session uses `dcp-integrations` from a terminal with the same commands as the provisional tool (poll, check, show, template, send, reply, record, decide, guide, status, defer, list, watch, projects, version), so the Python tool can be retired.

**Why this priority**: the provider is the engine behind every story above, and the projects' sessions need it without Aurora's window (ADR-0010).

**Independent Test**: Run the provider's integration tests on bare repositories: the scenarios of the provisional tool 0.3.0 (templates, round limit, ack outside rounds, closing record, decision, revision in cycle 2, poll without fetch, watch) pass on Windows and Linux.

**Acceptance Scenarios**:

1. **Given** a project folder with `.aurora/integrations.json`, **When** a command runs there, **Then** the provider takes the project's slug and its counterparts' repositories from that file (FEAT-029) and nowhere else.
2. **Given** `dcp-integrations init --project <slug>`, **When** it runs in a project folder, **Then** it writes `.aurora/integrations.json` with the project's own repository and no counterparts, for the user to complete and commit; `add <slug> --url <repository>` adds a counterpart and `validate` checks the file.
3. **Given** an `aurora/inbox` branch with `inbox.json`, **When** the provider reads it, **Then** it accepts schema `aurora.inbox` MAJOR 1 and refuses another MAJOR with a message naming both versions; a branch without `inbox.json` (written by the provisional tool) is read as MAJOR 1 and gets the file on the next write.
4. **Given** messages written by the provisional tool 0.1 to 0.3, **When** the provider reads them, **Then** they are shown unchanged (tool 0.1 messages, `kind: message`, are each their own conversation).
5. **Given** every command, **When** it runs with `--json`, **Then** it answers with a versioned document (`schema` and `schema_version`, contracts in `contracts/`), and without it with the same text as the provisional tool.

### Edge Cases

- A project registered in Aurora without `.aurora/integrations.json`: Messages shows it under "Not set up" with the `init` command to run; nothing else runs for it.
- A counterpart whose repository cannot be reached: its part of a conversation is shown as "not reachable" with the Git error to open; the rest still shows.
- Two projects of one conversation are both registered in Aurora: the conversation is listed once, under the opening project, with both sides' messages.
- A closing record whose opening project is not registered in Aurora: it is listed under the registered side as "waiting for <project> to present it", with no decide buttons.
- The user decides on a record while a new message arrives in the same thread: the provider's order (dates with milliseconds) decides; Aurora reads the conversation again after every action.
- Two quick presses on a send or decide button: one command runs; the button is disabled while it runs.
- A message body too large (over 256 KiB): refused in the form before anything runs.
- A provider missing, of an unsupported version, failing, timing out or answering invalid JSON: Messages says so with the error output to open, like the other engines; nothing is guessed.
- A Git push that loses a race is retried by the provider; Aurora only sees the final outcome.
- Aurora itself never runs Git, never fetches or pushes, and never writes an `aurora/inbox` branch or `.aurora/integrations.json`: the provider does (ADR-0007, ADR-0010).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The navigation MUST have a "Messages" entry with a count of conversations that have news or wait for the user.
- **FR-002**: Messages MUST show, in this order: closing records waiting for the user, conversations with a request from the user still being worked on, then every other conversation newest first; with filters by project, by "waiting for a project's answer" and by "no answer yet".
- **FR-003**: Every read and write MUST go through `dcp-integrations` with `--json`, without a shell, in the project's folder, with the user's time limit and the existing output size limits (ADR-0003); only when the provider reports schema MAJOR 1 for the commands used (`dcp-integrations version --json`).
- **FR-004**: The closing record MUST be shown as fields from `record --json`, never by parsing prose; decide offers the numbered options of "Decision needed" when they are numbered, and always a free choice.
- **FR-005**: "Ask for changes" and the conversation field MUST send the user's text through `decide --revise` and `guide`, which restart the rounds in a new cycle (ADR-0009).
- **FR-006**: New conversations and answers MUST be built from `template --json`; the core MUST refuse empty required sections, a kind not allowed at that point and a body over 256 KiB before anything runs; the provider stays the authority and its refusals are shown as returned.
- **FR-007**: While Aurora runs, it MUST poll at the interval set in Settings (default 5 minutes, "off" allowed), at most four projects at once, and read a project again only when its poll reports news.
- **FR-008**: The provider `dcp-integrations` MUST be a Rust binary built in this repository, with the commands, header fields, kinds, templates, round and cycle rules and closing record of the provisional tool 0.3.0, and reading every message that tool wrote.
- **FR-009**: The provider MUST take the project's identity and its counterparts only from `.aurora/integrations.json` (`init`, `add`, `validate`), write `inbox.json` (schema `aurora.inbox` MAJOR 1) on every branch it writes, and refuse an unknown MAJOR.
- **FR-010**: The provider MUST use Git through the `git` command with plumbing on bare caches, never touching a working tree or another branch, commit as the identity in Git's configuration of the project, and retry a push that loses a race.
- **FR-011**: The provider MUST be installed by Aurora's installers next to the desktop application on Windows and Linux, and published per platform as a separate download in the same release (ADR-0010).
- **FR-012**: The adapter MUST log each command's name, project, duration and outcome, never message bodies.
- **FR-013**: The browser preview MUST show Messages with sample conversations (a closing record waiting, a revision in cycle 2, an unanswered message, a project not set up) and simulate decide, revise, guide, send, reply and poll in memory.
- **FR-014**: A new feature "Messages" MUST appear in the Engines view, needing `dcp-integrations` MAJOR 1, release v0.3.0.

### Key Entities

- **Project inbox**: a registered project's slug, its counterparts and its messages, read through the provider.
- **Conversation**: a thread id, its projects, its cycle, its messages and whether a closing record waits for the user.
- **Message**: the header of ADR-0009 (id, kind, from, to, date, subject, status, thread, cycle, round, needs_user) and its sections.
- **Closing record**: the report for the user of an agreement or an impasse.
- **User action**: a decision, a request for changes, a request in a running conversation, a new message or an answer.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In the preview, the sample closing record is first, shows its six report fields, and a decision or a request for changes takes one action and one confirmation.
- **SC-002**: The provider's integration tests reproduce every scenario of the provisional tool 0.3.0 on bare repositories and pass on Windows and Linux.
- **SC-003**: Messages written by the provisional tool on the six Levix projects' inbox branches are read by the provider without change.
- **SC-004**: With nothing new, a poll of six projects finishes in under 5 seconds on a normal connection and fetches nothing.
- **SC-005**: No code path of Aurora runs Git or writes `.aurora/integrations.json` or an inbox branch; checked by review and by the adapter tests with a fake provider.
- **SC-006**: 100% of the automated tests pass on Windows and Linux with no provider installed (adapter tests use a fake).

## Assumptions

- The provider is a new crate of this workspace (`crates/aurora-integrations`, binary `dcp-integrations`), so it ships with Aurora's releases; its name follows the `dcp-<name>` convention of DCP FEAT-018 so it can plug into the Dev Cycle later (FEAT-037, v0.6.0) without renaming.
- Plugging into the Dev Cycle (`describe`, cycle points, integration gate) is out of scope here: FEAT-036 and FEAT-037, v0.6.0, after dcp v2.11.0.
- Merge notices written automatically (FEAT-030) are out of scope; a `notice` can be sent by hand.
- The Levix projects commit `.aurora/integrations.json` with their counterparts before switching from the provisional tool; until then the provisional tool keeps working, since both read and write the same branches.
- Signing the provider waits for FEAT-035, like the packages.
- Aurora's Messages decisions are the user's own: Aurora writes them through the provider only after the user's confirmation, never on its own.
