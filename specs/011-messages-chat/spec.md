# Feature Specification: Messages as a chat between projects

**Feature Branch**: `feat/011-messages-chat`
**Created**: 2026-11-08
**Status**: Draft, mockups shown to the user (2026-11-08)
**Input**: Backlog FEAT-039 (Levi, 2026-11-08). Mockups: `mockup-chat.png` (two columns) and `mockup-chat-detail.png` (a message opened).

The Messages view stops being a mailbox and becomes a chat between projects, laid out like a desktop messaging app: projects on the left as contacts, the selected project's conversation stream in the middle, and the full message in a panel that slides in on the right. The provider, the transport, the message format and the conversation rules stay as they are; the only change outside the view is an optional summary written by the sender.

## User Scenarios

### US1 - Pick a project like a contact (P1)

The user sees every registered project as a contact and opens its chat with one click.

**Acceptance**
1. The left column is narrow (about 300 px), never moves and never shrinks when the detail panel opens.
2. Each row shows the project's picture, its name, the last summary (prefixed `You:` when the project sent it) and the time of the last message (time today, `Yesterday`, weekday within a week, date before).
3. A row shows the number of unread received messages in a badge, and an amber dot when a conversation of the project waits for the user.
4. Rows are sorted by the last message, newest first; projects without messages come last with `No messages yet`.
5. The project picture is the logo set for the project in Aurora when there is one (a local image file chosen in the project's settings); otherwise a circle with the project's initials on a color derived from its slug.
6. A search box filters the rows by project name and by summary or subject text.

### US2 - Read the exchange as a chat (P1)

**Acceptance**
1. The middle column shows every message the selected project sent or received, oldest at the top, opening scrolled to the newest, with day separators.
2. Received messages are bubbles on the left with the sender's small picture and name in the sender's color; sent messages are bubbles on the right with another background and no name.
3. Each bubble shows the message's summary, not the body. A message without a summary shows its subject.
4. The first message of a conversation shows a small line above the text: subject, kind and round. A reply shows a quote of the message it answers (sender and the first words of its summary); a click on the quote scrolls to that bubble.
5. The bubble footer shows the kind, the time and the handling mark: `✓` sent and not handled yet, `✓✓` (accent) handled by the recipient, a clock for deferred. An unread received message is marked until it is seen.
6. The header shows the project's picture and name, `N active conversations · M wait for you`, and a filter `With: all projects ▾` to show only the messages exchanged with one counterpart.
7. New messages appear at the bottom as the worker refreshes the snapshot; the view stays where the user scrolled, with a `New messages ↓` button when the bottom is out of sight.

### US3 - Open the full message (P1)

**Acceptance**
1. A click on a bubble highlights it (accent outline) and a third column slides in on the right (about 440 px, 200-250 ms ease); the chat column shrinks to share the space; the project list does not move.
2. The panel shows what the opened conversation shows today for one message: from → to, subject, kind, date, round, the sender's summary in a box, the body with its sections, Next step callout, Authority and Open points folded, `Handled by <project> · date`, and the Reply and Step in buttons.
3. A click on another bubble replaces the content of the panel without closing it; Close (×) or Esc closes it and the chat takes the space back.
4. Opening a received message marks it as seen, as today.
5. Reply writes a reply to the opened message; the new message appears as a sent bubble at once (optimistic, through the existing write queue) and is rolled back with the provider's reason if refused.

### US4 - Decide inside the chat (P1)

**Acceptance**
1. A closing record (agreement or impasse) waiting for the user appears in the stream, centered, as the decision card that exists today: question, one button per option, Recommended marked, `Something else…`, the 5 s Undo.
2. Once decided, the card stays in the stream in a closed state (green) with the choice and its date; the decision message itself is not shown as a separate bubble.
3. A conversation closed with an impasse shows its card in the impasse color.

### US5 - Write the summary when sending (P1)

**Acceptance**
1. `dcp-integrations send` and `reply` accept `--summary "<text>"`: one or two sentences in the first person, as the sender speaking to the recipient; at most 280 characters.
2. The summary is written in the message's front matter as `summary`; the body is unchanged.
3. Without `--summary`, the message is written without the field and the chat shows the subject; nothing is generated.
4. `decide` and `guide` messages, written by the tool for the user, carry a summary made from their content (`Decision: <choice>`, `User request: <first sentence>`).
5. The `template` command and the project instructions mention the summary, so other projects start sending it.
6. `export`, `thread`, `check` and `show` return the summary when present; the export schema adds the optional field without a breaking change.

## Requirements

- **FR-001** The message front matter has an optional `summary` field (single line, at most 280 characters, quoted). Readers ignore it when missing; older tool versions keep reading messages that have it.
- **FR-002** `send` and `reply` take `--summary`; longer text is refused with a clear error, not cut.
- **FR-003** The export schema version goes up by one minor step and adds `summary` to each message item; the worker and the snapshot carry it.
- **FR-004** The core builds, per project, one chronological stream of its sent and received messages and closing records, with the quote reference (reply_to) and the handling mark for each message.
- **FR-005** The view has three columns: project list (fixed), chat (flexible), detail (closed by default, opened by a click on a bubble); the transition animates the column width and respects reduced motion.
- **FR-006** A project in Aurora may have a logo: an image file chosen in the project settings, copied into Aurora's data folder; the registry and the repositories are not changed.
- **FR-007** The colors of sent bubbles, received bubbles, the highlight and the separators are tokens in both themes and pass the contrast checks (WCAG 2.2 AA).
- **FR-008** The keyboard reaches every row, bubble and button; Up and Down move between bubbles, Enter opens the panel, Esc closes it.
- **FR-009** The mailbox view of v0.3.0 is replaced; its decision card, Handled chip, Next step callout and write queue are reused.

## Out of scope

- Gedo alerts between Gedo instances in the same style (FEAT-040, later).
- Generating summaries in Aurora or in the tool.
- Changes to the transport, to the conversation rules (rounds, records, decisions) or to other projects' repositories.

## Success Criteria

- **SC-001** The user can tell who said what in a conversation by reading only the bubbles, without opening any message.
- **SC-002** Opening and switching the detail panel takes under 100 ms from the snapshot; the slide finishes in under 250 ms.
- **SC-003** Messages without a summary (all existing ones) display correctly with their subject.

## Open questions

- Release: v0.3.0 before the final tag, a v0.3.x right after it, or v0.4.0 (asked to the user).
