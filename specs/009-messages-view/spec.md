# Feature Specification: Messages view adjustments

**Feature Branch**: `feat/009-messages-view`

**Created**: 2026-11-06

**Status**: Approved (gate 3, 2026-11-06)

**Input**: User description: "Testing v0.3.0-rc.1, the Messages view should say clearly who sent each message to whom (From → To), keep the messages together by conversation, show the closed conversations in a different color from those in progress, show the number of open messages next to Messages in the navigation on every screen, and open a conversation right below the clicked row instead of at the bottom of the page. Agreed: three groups (Waiting for you, In progress, Closed, the last muted and collapsed), inline opening rather than a popup, and the count = conversations waiting for the user plus those with received messages not read yet. Backlog item ADJ-002, release v0.3.0 (Levi, 2026-11-06)."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - See who talks to whom (Priority: P1)

The developer reads the conversation list and each message and knows at once who sent it and to whom, without reading the body.

**Why this priority**: the first thing Levi missed in the release candidate.

**Independent Test**: In the preview, open Messages and check that every row and every message header starts with `sender → receiver`.

**Acceptance Scenarios**:

1. **Given** a conversation, **When** the list shows it, **Then** its row starts with the last message's `from → to` (slugs), then the subject, the opener, the projects in it and the number of messages.
2. **Given** an opened conversation, **When** its messages are shown, **Then** each header starts with `from → to`, then the kind, cycle, round and date.

---

### User Story 2 - Tell the state of each conversation at a glance (Priority: P1)

The list is split into three groups, each with its color: what waits for the user, what is still in progress, and what is closed.

**Why this priority**: closed and running conversations look the same today.

**Independent Test**: In the preview, check the three groups in order, the Closed group collapsed, and each sample conversation in the right group.

**Acceptance Scenarios**:

1. **Given** a conversation whose closing record waits for the user, **When** the list is shown, **Then** it is in **Waiting for you**, in the warning color.
2. **Given** a conversation whose current cycle has no decision yet and no record waiting, **When** the list is shown, **Then** it is in **In progress**, in the normal color.
3. **Given** a conversation whose last message is a decision, or that holds only notices and acks, **When** the list is shown, **Then** it is in **Closed**, muted, and the group is collapsed until the user opens it.
4. **Given** a revision after a decision (cycle 2), **When** the list is shown, **Then** the conversation is back in **In progress**.
5. **Given** a group with no conversation, **When** the list is shown, **Then** the group is not shown, except Waiting for you, which says "Nothing waits for your decision".

---

### User Story 3 - Read a conversation where it was clicked (Priority: P1)

Clicking a row opens the conversation right below it; clicking it again, or Escape, closes it.

**Why this priority**: today the conversation opens at the bottom of the page, far from the row.

**Independent Test**: In the preview, open the second row and check the conversation appears between it and the next row, that only one is open at a time, and that Escape closes it and returns the focus to the row.

**Acceptance Scenarios**:

1. **Given** the list, **When** the user activates a row, **Then** the conversation opens right below it, the row shows it is expanded, and the focus moves into the conversation.
2. **Given** an open conversation, **When** the user activates another row, **Then** the first closes and the second opens below its own row.
3. **Given** an open conversation, **When** the user presses Escape or activates its row again, **Then** it closes and the focus returns to its row.

---

### User Story 4 - See what is open from any screen (Priority: P2)

Next to Messages in the navigation, a number shows the conversations that need the user, on every screen.

**Why this priority**: the user works in other views and must notice new messages.

**Independent Test**: In the preview, stay on Portfolio and check the count next to Messages equals the sample's conversations waiting for the user plus those with unread received messages; after the preview's poll fires, the count goes up.

**Acceptance Scenarios**:

1. **Given** Aurora starts, **When** the messages are read, **Then** the count is the number of conversations that wait for the user or have received messages not read yet, across every project, and it is hidden at 0.
2. **Given** a poll with news, **When** it answers, **Then** the count is recomputed on whichever screen is shown.
3. **Given** a conversation is opened and its unread messages are marked seen, **When** the list is read again, **Then** the count goes down.

### Edge Cases

- A conversation the user is in on both sides (two registered projects) is counted once per project that lists it; the count follows the list.
- Messages of tool 0.1 (kind `message`) are their own conversation and count as In progress while open, Closed once done.
- Unread is what the provider reports as received and open; a provider that does not report it gives 0, never an error.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Rows and message headers MUST lead with `from → to` (slugs).
- **FR-002**: Rows MUST show the subject, the opener, the projects and the number of messages of the conversation.
- **FR-003**: The list MUST be grouped in Waiting for you, In progress and Closed, in that order, each with its own color; Closed MUST be collapsed by default.
- **FR-004**: The state MUST be derived in the core from the conversation (waiting for the user; last message a decision or only notices and acks; otherwise in progress), never in the view.
- **FR-005**: A conversation MUST open right below its row, one at a time, with `aria-expanded` on the row, focus moved into it, and Escape closing it.
- **FR-006**: The navigation count MUST be the conversations waiting for the user plus those with unread received messages, computed at start and after every poll, shown on every screen.
- **FR-007**: Unread MUST come from the received open messages that `dcp-integrations check --json` already returns; no provider or contract change.

### Key Entities

- **Conversation state**: Waiting for you, In progress or Closed, derived from the conversation.
- **Unread**: the number of received messages of a conversation still open for the project.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In the release candidate, Levi tells sender and receiver of every row without opening it.
- **SC-002**: Opening a conversation never scrolls the page away from the clicked row.
- **SC-003**: The count next to Messages matches the list on every screen.

## Assumptions

- The provider's `check` keeps listing received open messages in `messages` (contract `dcp-integrations.check` 1.0).
- A decision closes the current cycle; a later guidance or revision opens the next one.
