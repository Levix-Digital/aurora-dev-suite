---
id: 13
title: Messages read as a chat between projects, with a summary written by the sender
status: accepted
date: '2026-11-08'
version: v0.3.1
subsystems: [ui, core, integrations, desktop]
supersedes: []
superseded_by: []
amends: [12]
amended_by: []
---

# ADR-0013: Messages read as a chat between projects, with a summary written by the sender

## Context

On 2026-11-08 Levi asked for the Messages view to read like a desktop messaging app: the projects as contacts, a chat of what each project sent and received, and the full message only when asked for. The mailbox of ADR-0012 lists conversations and opens them as e-mail threads; reading who said what in a conversation still means opening each message. Messages have no short text written for a reader at a glance: the subject repeats `Re:` and the body is in sections. Levi decided (mockups agreed the same day):

- two columns, projects (narrow, never moving) and chat (wide); a click on a message slides a third column in on the right with the full message, as it shows today;
- received messages on the left, sent ones on the right in another style, each showing a short first-person summary;
- the summary is written by the sender; older messages show the subject;
- closing records are cards in the chat, like the decision card;
- Aurora's palette only.

## Decision

- **Summary in the message.** The front matter gets an optional `summary` field: one or two sentences in the first person, at most 280 characters, written after the fixed fields only when set, so every other message is written exactly as before. `send` and `reply` take `--summary`; a longer summary is refused, never cut. `decide` and `guide`, written by the tool for the user, make their own ("The user decided: …", "The user asks: …"). Every header in the replies carries `summary`; documents move to schema 1.3 and `version` declares the capability `summary`. Nothing generates summaries.
- **A chat model in the core.** `domain::chat` reads the snapshot's inboxes (ADR-0012 is unchanged): `chat_contacts` gives one row per registered project with its last summary, time, unread count and waiting records; `chat_stream` gives one project's messages, oldest first, with day separators in the user's time zone, the quote of the message a reply answers (when it is not the bubble right above), a delivery mark from the receiver's status (sent, read, handled, set aside) and the closing record as a card after the message that closed. Decision messages are not bubbles: they close their card.
- **Three columns.** The view keeps the snapshot, the write queue, the decision card, the message body and the forms of ADR-0012, and lays them out as contacts, chat and a detail panel that animates its width (none with reduced motion). The chat follows one project; a counterpart filter shows one exchange.
- **Project logo.** A project may have a logo: a PNG, JPEG, WebP or SVG of at most 256 KB, picked in the Portfolio and kept as a `data:` URL in Aurora's own project list. Without one, the initials on a palette color derived from the name. Nothing is written in the repositories or in `.aurora/integrations.json`.

## Alternatives Considered

- **Aurora writes the summary** (first line of the body, or generated): the lead sentence is often not a summary, and generating text would need a model inside an air-gapped app; the sender knows best what it means.
- **A chat per pair of projects** (a contact per counterpart): with several registered projects the same pair appears twice; following one project and filtering by counterpart covers both readings.
- **Keep the mailbox and add a summary line**: still one row per conversation, not a dialogue between projects, which is what was asked for.
- **Logo in the repository** (a file referenced by `.aurora/integrations.json`): would change other projects' registries and repositories; the logo is Aurora's presentation only.

## Consequences

- Other projects start sending summaries when their instructions and tools pass `--summary`; until then their bubbles show subjects.
- Older providers ignore nothing: a message with `summary` stays readable by tool 0.3 and earlier, which ignore unknown header fields.
- The mailbox tabs, status filter and legend of spec 010 leave the view; the navigation count and the states stay.
