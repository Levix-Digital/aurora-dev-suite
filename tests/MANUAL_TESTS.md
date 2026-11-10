# Manual Tests

Curated manual test suite: one section per feature, each test describing how the software behaves **today**. Tests are edited in place when behavior changes and removed when they no longer apply. Only what cannot be automated is tested here.

How to record a run: in each test of the current run, mark exactly one box of the `Result` line with `x` and, for `Fail` or `Blocked`, describe what happened in `Notes`.

Building Aurora for these tests (from the repository root):

```sh
cd crates/aurora-ui && trunk build --release && cd ../..
cargo build -p aurora-desktop --release --features custom-protocol
```

The binary is `target/release/aurora` (`aurora.exe` on Windows). A fake engine for the engine tests is built with `cargo build -p aurora-engines --features fake-engine` (`target/debug/aurora-fake-engine`).

## Foundations: window and navigation

### MT-WIN-001: The window opens on Windows and on Linux

- **Covers**: desktop shell, Portfolio view (FR-001, SC-001)
- **Prerequisites**: a release build on Windows 10 22H2 or later, or on Linux with WebKitGTK 4.1
- **Steps**:
  1. Start `aurora` (or `aurora.exe`) and count the time until the window is usable
- **Expected**:
  - The window titled "Aurora" appears within 1.5 seconds with the navigation and the Portfolio view selected
  - No "Preview" banner is shown
- **Result** (v0.3.0-rc.1): [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

### MT-WIN-002: Every view is reachable with the keyboard only

- **Covers**: navigation, focus styles (FR-002, SC-005)
- **Prerequisites**: Aurora open
- **Steps**:
  1. Press Tab until the focus is in the navigation; use the Down and Up arrows to move between the items and Enter to open each one
  2. Press Alt+1 to Alt+6
  3. In each view, press Tab through every control
- **Expected**:
  - The focused element always has a visible outline
  - Alt+1 to Alt+6 open Portfolio, Project, Engines, Inbox, Messages and Settings, with the focus in the content of the view
  - Every button and field can be reached and used without the mouse
- **Result** (v0.3.0-rc.1): [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

### MT-WIN-003: The theme follows the system and the setting

- **Covers**: theme, Settings view (FR-003)
- **Prerequisites**: Aurora closed; the system set to dark appearance
- **Steps**:
  1. Start Aurora
  2. In Settings, choose Light and Save; close and start Aurora again
  3. Choose "Follow the system" and Save, then switch the system to light appearance
- **Expected**:
  - Step 1 shows the dark theme; step 2 shows light after Save and after the restart; step 3 follows the system change without a restart
- **Result** (v0.3.0-rc.1): [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

### MT-WIN-005: Linux without WebKitGTK

- **Covers**: start-up on Linux (edge case)
- **Prerequisites**: a Linux machine or container without `libwebkit2gtk-4.1-0`
- **Steps**:
  1. Run `aurora` from a terminal
- **Expected**:
  - The terminal names the missing library (`libwebkit2gtk-4.1.so.0`); the README's troubleshooting section tells which package to install
- **Result** (v0.3.0-rc.1): [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

### MT-WIN-006: The log file is written

- **Covers**: observability (plan)
- **Prerequisites**: Aurora started once
- **Steps**:
  1. Open the log folder: `~/.local/share/aurora/logs` on Linux, `%LOCALAPPDATA%\Levix\Aurora\data\logs` on Windows
  2. Start Aurora with `AURORA_LOG=debug` and open the Engines view
- **Expected**:
  - A file `aurora.<date>.log` has a "starting Aurora" line and one "engine probed" or "engine missing" line per engine; with `debug`, each engine command and its duration; never the engine's output
- **Result** (v0.3.0-rc.1): [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

## Foundations: projects

### MT-PRJ-001: Add projects with the folder dialog and keep them after a restart

- **Covers**: Portfolio, `pick_and_add_project` (FR-005, FR-008, SC-002)
- **Prerequisites**: three local Git repositories
- **Steps**:
  1. Click "Add a project…" and choose the first repository; repeat for the others, choosing a sub-folder for the third
  2. Rename one, then close and start Aurora again
- **Expected**:
  - Each appears with its folder name; the third says its repository root was registered instead of the sub-folder
  - After the restart the list, with the new name, is the same
  - Done in under one minute
- **Result** (v0.3.0-rc.1): [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

### MT-PRJ-002: Refused folders say why

- **Covers**: Portfolio (FR-005, FR-007)
- **Prerequisites**: one registered repository
- **Steps**:
  1. Type a folder outside any repository and click Add
  2. Type a path that does not exist and click Add
  3. Type the registered repository again through another path (a link, or other letter case on Windows) and click Add
- **Expected**:
  - Nothing is added; the messages say "not inside a Git repository", "does not exist" and "already registered as …"
- **Result** (v0.3.0-rc.1): [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

### MT-PRJ-003: A moved project is shown as missing and can be located

- **Covers**: Portfolio, `relocate_project` (FR-009)
- **Prerequisites**: one registered repository
- **Steps**:
  1. Close Aurora, move the repository's folder, start Aurora
  2. Click "Locate…", type the new path and Save
- **Expected**:
  - Step 1 shows the project as "Missing" with its old path; step 2 shows it as "Present" at the new path
- **Result** (v0.3.0-rc.1): [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

### MT-PRJ-004: Removing a project never touches the repository

- **Covers**: Portfolio, `remove_project` (FR-006)
- **Prerequisites**: one registered repository
- **Steps**:
  1. Click Remove, then confirm
- **Expected**:
  - The project leaves the list; the repository folder is unchanged on disk
- **Result** (v0.3.0-rc.1): [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

### MT-PRJ-005: A damaged project list is kept aside

- **Covers**: registry recovery (FR-010)
- **Prerequisites**: Aurora closed; `projects.toml` in the configuration folder (`~/.config/aurora` on Linux, `%APPDATA%\Levix\Aurora\config` on Windows)
- **Steps**:
  1. Replace the content of `projects.toml` with `not [valid`
  2. Start Aurora
- **Expected**:
  - The Portfolio is empty and a notice gives the path of `projects.toml.damaged-<timestamp>`, which holds the old content
- **Result** (v0.3.0-rc.1): [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

## Foundations: engines

### MT-ENG-001: No engine installed

- **Covers**: Engines view (US3 scenario 1)
- **Prerequisites**: none of `dcp`, `gedo`, `actx` on the search path
- **Steps**:
  1. Open Engines
- **Expected**:
  - The three engines are "Missing" with what to install; every feature says what it needs; Aurora keeps working
- **Result** (v0.3.0-rc.1): [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

### MT-ENG-002: Refresh after installing an engine

- **Covers**: `refresh_engines` (FR-016, SC-004)
- **Prerequisites**: Aurora open on Engines; the fake engine built
- **Steps**:
  1. Copy `aurora-fake-engine` as `gedo` into a folder on the search path (it answers the contract by default) and click Refresh
  2. Write `no-contract` into a file `gedo.mode` next to it and click Refresh
- **Expected**:
  - Step 1: Gedo is "Ready" with version 1.2.3 and its capabilities, and "Agent work" shows "Coming in v0.4.0"
  - Step 2: Gedo is "Installed without contract" with what to update
  - On Windows, no console window opens at any step
- **Result** (v0.3.0-rc.1): [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

### MT-ENG-003: A hanging engine only delays itself

- **Covers**: parallel probes, time limit (FR-015, SC-003)
- **Prerequisites**: fake engines installed as `gedo` (mode `hang`) and `actx` (mode `ready`); Gedo's time limit set to 3 seconds in Settings
- **Steps**:
  1. Click Refresh on Engines
- **Expected**:
  - AnyContext updates at once; Gedo turns "Degraded" ("no answer after 3 s") after about 3 seconds; "Last error output" opens
- **Result** (v0.3.0-rc.1): [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

### MT-ENG-004: A configured location wins over the search path

- **Covers**: Settings, probe (FR-011)
- **Prerequisites**: a fake engine copied as `gedo` into a folder that is not on the search path
- **Steps**:
  1. In Settings, set the Gedo location to that file and Save; refresh Engines
- **Expected**:
  - Gedo is "Ready" and its location is the configured file
- **Result** (v0.3.0-rc.1): [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

## Foundations: preview

### MT-PRV-001: The interface runs alone in a browser

- **Covers**: fixture backend, preview banner (FR-017, FR-018)
- **Prerequisites**: Trunk 0.22 installed
- **Steps**:
  1. Run `trunk serve` in `crates/aurora-ui` and open `http://127.0.0.1:1420`
  2. Visit every view; on Engines, click Refresh twice
- **Expected**:
  - A "Preview" banner is shown; the Portfolio lists four sample projects, one missing
  - The engines show ready, installed without contract and missing; after one refresh Gedo is degraded, after the next ready again
- **Result** (v0.3.0-rc.1): [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

## Portfolio on the dcp contract

### MT-POR-001: The preview shows each project's cycle summary

- **Covers**: Portfolio rows, phase and waiting items (spec 003 US1, SC-001)
- **Prerequisites**: Trunk 0.22 installed
- **Steps**:
  1. Run `trunk serve` in `crates/aurora-ui`, open `http://127.0.0.1:1420` and look at the Portfolio
- **Expected**:
  - `gedo`: "Implementation", "v0.4.0 Command line", "3 wait for you: 1 pull request, 1 blocking bug, 1 untriaged bug"
  - `any-context`: "Idle", "No roadmap", "Nothing waits for you"
  - `ai-code-agent-skills`: "Release candidate", "v2.9.0 Baseline and zero friction", "1 waits for you: 1 release candidate" and "Part of the project could not be read."
  - `levix-platform`: "Missing" and "Locate the folder to read its cycle state."
  - The header says "4 items wait for you"
- **Result** (v0.3.0-rc.1): [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

### MT-POR-002: Without dcp the Portfolio says what to install once

- **Covers**: unavailable state (spec 003 US3, SC-004)
- **Prerequisites**: no `dcp` on the search path and no `dcp` location in Settings; at least two projects registered
- **Steps**:
  1. Start Aurora and look at the Portfolio
- **Expected**:
  - The Portfolio line at the top says to install the Dev Cycle Protocol with its JSON contract
  - Every present project says "Cycle state unavailable." within a second; missing projects say to locate the folder
- **Result** (v0.3.0-rc.1): [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

### MT-POR-003: With dcp answering the contract, every project shows its summary

- **Covers**: Portfolio over the dcp adapter (spec 003 US1)
- **Prerequisites**: the fake engine set as the `dcp` location in Settings (it answers the contract by default); at least two present projects registered
- **Steps**:
  1. Refresh the engines, then open the Portfolio and click Refresh
- **Expected**:
  - Every present project shows "Implementation", "v0.4.0 Command line" and "3 wait for you"
  - While reading, the rows say "Reading the cycle state…" or "Refreshing…" and the header counts the reads
- **Result** (v0.3.0-rc.1): [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

### MT-POR-004: A hanging dcp only delays its own rows

- **Covers**: parallel reads, failures per row, Refresh (spec 003 US2, SC-002)
- **Prerequisites**: as MT-POR-003, with the `dcp` time limit set to 3 seconds in Settings
- **Steps**:
  1. Write `hang` into a file `dcp.mode` next to the fake engine and click Refresh on the Portfolio
  2. Remove `dcp.mode` and click Refresh again
- **Expected**:
  - Step 1: each row keeps its last summary marked "Refreshing…", then says the Dev Cycle Protocol did not answer within 3 s; the window stays responsive and the project actions keep working
  - Step 2: the summaries come back
- **Result** (v0.3.0-rc.1): [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

## Project view

### MT-PVW-001: Open a project from the Portfolio and see its whole cycle

- **Covers**: Open, sections of the cycle state (spec 004 US1, SC-001)
- **Prerequisites**: Trunk 0.22 installed
- **Steps**:
  1. Run `trunk serve` in `crates/aurora-ui`, open `http://127.0.0.1:1420` and press **Open** on `gedo`
- **Expected**:
  - The Project view opens with the title `gedo` and the project choice set to `gedo`
  - Overview: "Implementation", "v0.4.0 Command line", "3 wait for you: 1 pull request, 1 blocking bug, 1 untriaged bug", the engine's summary line and the read time, as in the Portfolio row
  - Plans: "Command line (in_progress)" with a bar and "3 of 5 milestones"
  - Roadmap: "v0.4.0 Command line" with "FEAT-031 (high) gedo radar --json" and "FEAT-034 (medium) gedo doctor --json"
  - Release candidate: "No release candidate"
  - Pull requests: "#41 feat: gedo radar --json", "Needs your action", "2 threads open, CI green. 1 review thread for you"
  - Blocking bugs: "BUG-012 (high) Merge falls back to text on renamed files"; untriaged bugs: "BUG-015 Slow radar on large repositories: no severity"
- **Result** (v0.3.0-rc.1): [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

### MT-PVW-002: Gates and the next step

- **Covers**: gates, next step order (spec 004 US2)
- **Prerequisites**: as MT-PVW-001
- **Steps**:
  1. With `gedo` open, look at Gates and Next step
  2. Choose `ai-code-agent-skills` in the project choice
  3. Choose `any-context`
- **Expected**:
  - Step 1: the gate "Decision" says "Choose how to go on: Fix the blocking bug first"; Next step lists "Fix the blocking bug first" first, marked "Recommended", then "Resume plan command-line" and "Answer the review of pull request 41", each with its reason when it has one
  - Step 2: "Release candidate" phase; the gates "Decision" and "Release candidate acceptance" with "v2.9.0-rc.1: 4 not run, 1 failed"; the release candidate "v2.9.0-rc.1 of v2.9.0: 4 not run, 1 failed" and "Not released"; the overview lists the finding about pull requests
  - Step 3: "Idle", "No gate waits for you", "No step recommended", and every other section says it has nothing
- **Result** (v0.3.0-rc.1): [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

### MT-PVW-003: Switching, a missing project and Refresh

- **Covers**: project choice, failures, Refresh (spec 004 US3, SC-002)
- **Prerequisites**: as MT-PVW-001, reloaded so nothing is chosen
- **Steps**:
  1. Open the Project view with Alt+2
  2. Choose `levix-platform`
  3. Choose `gedo`, press **Refresh**
  4. Switch to the Portfolio and back to the Project view
- **Expected**:
  - Step 1: the first registered project is shown
  - Step 2: "Missing: folder moved or deleted" and "Locate the folder to read its cycle state.", the same text as its Portfolio row
  - Step 3: the sections stay while "Refreshing…" is shown, then the state comes back
  - Step 4: `gedo` is still chosen
- **Result** (v0.3.0-rc.1): [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

### MT-PVW-004: Without dcp the Project view says what to install once

- **Covers**: unavailable state (spec 004 US3, SC-003)
- **Prerequisites**: no `dcp` on the search path and no `dcp` location in Settings; at least one present project registered
- **Steps**:
  1. Start Aurora, press **Open** on a present project
- **Expected**:
  - The Project view line at the top says to install the Dev Cycle Protocol with its JSON contract
  - The view says "Cycle state unavailable." within a second
- **Result** (v0.3.0-rc.1): [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:


## Project settings

### MT-SET-001: The preview shows a project's settings by group

- **Covers**: Settings tab, rendering from `dcp config get` (spec 007 US1, SC-001)
- **Prerequisites**: Trunk 0.22 installed
- **Steps**:
  1. Run `trunk serve` in `crates/aurora-ui`, open `http://127.0.0.1:1420`, open Project on `gedo` and press the **Settings** tab
- **Expected**:
  - The tabs "Cycle" and "Settings" are shown; "Source: .dev-cycle.json"
  - The groups `devops`, `architecture`, `backlog`, `docs`, `speckit`, `plans` and `principles`, in this order
  - "Branching model" is a choice set to `trunk` with "Default: integration"; "Release candidates" is a checked box; "Bug severities that block new work" has `critical`, `high`, `medium` checked and `low` not; "README line limit" is `400`
  - "Integration branch", "Production branch" and "Release candidates" are marked "Default"; "Branching model" and "Latest release for a bug that does not block" are not, and have a **Reset to default** button
  - "Mandatory gates", the two `speckit` settings and "Rules that flag a change drifting from the active plan" show their value as text with no control, marked "Read-only" with their reason (for example "Changed only by dcp speckit init and dcp speckit upgrade"); the drift rules show `[{"path":"crates/gedo-core/**","plan":"required"}]`
- **Result** (v0.3.0-rc.1): [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

### MT-SET-002: Edit, preview and apply a change

- **Covers**: draft, `preview_project_config`, `apply_project_config` (spec 007 US2 scenarios 1 to 4)
- **Prerequisites**: the preview of MT-SET-001
- **Steps**:
  1. Set "Branching model" to `integration` and clear "Release candidates"
  2. Press **Preview changes**, then **Back to editing**, then **Preview changes** again
  3. Press **Apply**
- **Expected**:
  - Step 1: both fields are marked "Edited" and the toolbar says "2 changes"
  - Step 2: the preview lists `devops.branching: trunk → integration` and `devops.release_candidates: on → off` and shows the diff of `.dev-cycle.json`; back to editing keeps both edits
  - Step 3: "Saved to .dev-cycle.json. Commit it to share the change."; the settings are read again with the new values and "No changes"
- **Result** (v0.3.0-rc.1): [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

### MT-SET-003: Refusals are shown next to the field

- **Covers**: the core's checks and the engine's refusals (spec 007 US2 scenario 5, FR-006, FR-007)
- **Prerequisites**: the preview of MT-SET-001
- **Steps**:
  1. Type `abc` in "README line limit" and leave the field
  2. Type `400` back; set "Integration branch" to `main` and press **Preview changes**
  3. Press **Discard**
- **Expected**:
  - Step 1: "Expected a whole number" under the field; the toolbar does not count it
  - Step 2: "The integration branch must differ from the production branch" under "Integration branch" and in the status line; nothing is written
  - Step 3: the edits and the errors are gone
- **Result** (v0.3.0-rc.1): [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

### MT-SET-004: Without the config capability nothing runs

- **Covers**: availability (spec 007 US3, SC-003)
- **Prerequisites**: the desktop build without `dcp` on the search path, then with the fake engine copied as `dcp` and mode `ready` (it lists `diagnose` and `config`)
- **Steps**:
  1. Without `dcp`, open Project, choose a project and press **Settings**
  2. Put the fake `dcp` on the search path, refresh the Engines view and open **Settings** again; change "Branching model" and preview it
- **Expected**:
  - Step 1: "Project settings" says what to install (needs config) and "Project settings unavailable."; no process is started
  - Step 2: the sample settings are shown and the preview lists the sample changes; the project's `.dev-cycle.json` is not touched by Aurora
- **Result** (v0.3.0-rc.1): [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

### MT-SET-005: Only what the user changed is written

- **Covers**: keys set in the file versus defaults, reset to default (spec 007 FR-003, FR-006, ADJ-001)
- **Prerequisites**: the preview of MT-SET-001
- **Steps**:
  1. Clear "Release candidates" and check it again
  2. Press **Reset to default** on "Folder of the core, which must not import adapters", then **Preview changes**
  3. Press **Apply**
- **Expected**:
  - Step 1: "Release candidates" is not marked "Edited" and the toolbar says "No changes" (its default is not written as a decision)
  - Step 2: the field is marked "Back to default" and shows "Not set"; the preview lists `architecture.core_path: crates/gedo-core → Not in the file (default)` and the diff removes the line
  - Step 3: after reading again, the field is marked "Default" and has no **Reset to default** button
- **Result** (v0.3.0-rc.1): [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

## Inbox

### MT-INB-001: Everything that waits, most urgent first

- **Covers**: Inbox items, urgency, order and details (spec 005 US1, SC-001, SC-002)
- **Prerequisites**: Trunk 0.22 installed
- **Steps**:
  1. Run `trunk serve` in `crates/aurora-ui`, open `http://127.0.0.1:1420` and note the Portfolio header's total
  2. Open the Inbox with Alt+4
- **Expected**:
  - Step 1: "4 items wait for you"
  - Step 2: "4 items wait for you across 2 projects", then the groups in this order, each with its count:
    - "Blocks a release (1)": `gedo`, "Blocking bug", "BUG-012 Merge falls back to text on renamed files", "Severity high"
    - "Waits for your acceptance (1)": `ai-code-agent-skills`, "Release candidate", "Run the manual tests of v2.9.0-rc.1", "4 not run, 1 failed"
    - "Waits for your review (1)": `gedo`, "Pull request", "#41 feat: gedo radar --json", "2 threads open, CI green. 1 review thread for you"
    - "To triage (1)": `gedo`, "Untriaged bug", "BUG-015 Slow radar on large repositories", "no severity"
  - The note that part of a project could not be read is shown (the release candidate sample has a finding)
- **Result** (v0.3.0-rc.1): [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

### MT-INB-002: Open the project of an item

- **Covers**: "Open" (spec 005 US2)
- **Prerequisites**: as MT-INB-001
- **Steps**:
  1. In the Inbox, press **Open** on "Run the manual tests of v2.9.0-rc.1"
  2. Go back to the Inbox and press **Open** on `levix-platform` under "Not read"
- **Expected**:
  - Step 1: the Project view opens on `ai-code-agent-skills`, with the gate "Release candidate acceptance"
  - Step 2: the Project view opens on `levix-platform`, which says to locate its folder
- **Result** (v0.3.0-rc.1): [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

### MT-INB-003: Refresh and the projects not read

- **Covers**: Refresh, "Not read" (spec 005 US3)
- **Prerequisites**: as MT-INB-001
- **Steps**:
  1. Look at the end of the Inbox
  2. Press **Refresh**
- **Expected**:
  - Step 1: "Not read (1)" lists `levix-platform` with "Locate the folder to read its cycle state.", the same text as its Portfolio row, and **Open**
  - Step 2: the items stay while the header says how many projects are being read, then the same items come back in the same order
- **Result** (v0.3.0-rc.1): [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

### MT-INB-004: Without dcp the Inbox says what to install once

- **Covers**: unavailable state (spec 005 US3, SC-003)
- **Prerequisites**: no `dcp` on the search path and no `dcp` location in Settings; at least two present projects registered
- **Steps**:
  1. Start Aurora and open the Inbox
- **Expected**:
  - The line at the top says to install the Dev Cycle Protocol with its JSON contract
  - "Nothing waits for you", and "Not read" says "Cycle state unavailable." once, within a second, not once per project
- **Result** (v0.3.0-rc.1): [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

## Messages

These tests need two projects registered in Aurora, each with `.aurora/integrations.json` listing the other (`dcp-integrations init` and `add`), and push access to both repositories. A conversation can be prepared from a terminal with `dcp-integrations` in either folder.

### MT-MSG-001: Decide on a closing record in the chat

- **Covers**: decision card in the stream (spec 008 US1, FR-001, FR-004; spec 010 US3; spec 011 US4)
- **Prerequisites**: a conversation between the two projects closed with an agreement (`reply <id> --kind agreement`) whose record has `Recommended: 1`, and no decision yet
- **Steps**:
  1. Open Messages (Alt+5) and pick the project that opened the conversation (amber dot on its row)
  2. Click option 2, then press Undo within five seconds
  3. Click option 1 and wait five seconds
  4. Pick the other project; in a terminal, run `dcp-integrations check` in its folder
- **Expected**:
  - Step 1 shows, right after the agreement bubble, the decision card centered in the chat: "Your decision · <subject> · agreement", the question, one button per option with option 1 marked Recommended, "Something else…", and the record folded under "Conclusion, positions and what follows"
  - Step 2 shows "sending … in 5 s" with a shrinking bar; Undo brings the options back and nothing is sent
  - Step 3 sends option 1 without any confirmation dialog; "Decision sent" appears, the card closes in green with the choice, the amber dot leaves the row and the navigation count drops; no separate bubble for the decision message appears
  - Step 4: the other project's chat shows the record closed with the same choice; `check` lists the decision as a new message
- **Result** (v0.4.0): [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

### MT-MSG-002: Decide in your own words, ask for a rework and step in

- **Covers**: own choice, revision and guidance (spec 008 US2, FR-005; spec 010 US2, US3; spec 011 US3, US4)
- **Prerequisites**: two closing records waiting, and one other conversation open
- **Steps**:
  1. On the first card, click "Something else…", leave the box empty and try to send; then write a decision and send it
  2. On the second card, click "Something else…", write a request and click "ask the projects to rework it"
  3. Click a bubble of the other conversation, press "Step in" in the panel on the right, write a request and send
- **Expected**:
  - With the box empty, nothing is sent and the card says what is missing
  - Step 1 sends your words as the decision (`decide --choice`); the card closes with your words
  - Step 2 sends a guidance message that starts cycle 2: a sent bubble "The user asks: …" appears in the chat, and the next bubble shows round 1
  - Step 3: the Reply and Step in forms stay closed until their button is pressed; the request appears as a sent bubble and the next reply counts from round 1 of the new cycle
- **Result** (v0.4.0): [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

### MT-MSG-003: Read the exchange as a chat

- **Covers**: contacts, bubbles and the detail panel (spec 011 US1, US2, US3)
- **Prerequisites**: across the two projects, a conversation waiting for the user, one with a received message not read, one closed by a decision, one message handled by its receiver, and messages on at least two different days; some messages sent with `--summary`, older ones without
- **Steps**:
  1. Open Messages and look at the list of projects and at the navigation
  2. Pick a project; scroll the chat to the top and back
  3. Click a received bubble with "● new", then another bubble, then press Escape
  4. Click the quote on top of a reply
  5. With more than one counterpart, choose "With: <project>", then "With: all projects"
  6. Type part of a subject in the search box above the projects
  7. Use the Tab key to reach a bubble, then the Up and Down arrows
- **Expected**:
  - Step 1: each registered project is a row with its picture (logo or initials on a palette color), its name, the last summary (`You: …` when it sent it), the time (`09:23`, `Yesterday`, a weekday or a date) and a badge with its unread count; an amber dot marks a project with a decision waiting; rows with the newest messages come first; the navigation shows "N active"
  - Step 2: received bubbles are on the left with the sender's small picture and name, sent bubbles on the right in the accent-tinted color; each shows the sender's summary, or the subject when the message has none; the first bubble of a conversation shows "subject · kind · round"; day separators read `Today`, `Yesterday` or `Saturday, 1 November`; sent bubbles end with ✓ (not read), ✓✓ (read) or ✓✓ in the accent color (handled); the chat opens at the newest message
  - Step 3: the bubble gets an outline and a panel slides in on the right with the full message: from → to, subject, kind, date and round, "Summary by the sender", the lead sentence, the sections, Next step in an amber callout, Authority and Open points folded, "✓ Handled by …", and Reply (for a received message) and Step in; the chat shrinks and the project list does not move; "● new" and the unread badge drop at once; the second click replaces the panel's content; Escape closes the panel and the chat takes the space back
  - Step 4 scrolls the chat to the quoted message
  - Step 5 shows only the messages with that project, then all again
  - Step 6 keeps only the projects whose name, subjects or summaries match
  - Step 7 moves the focus from bubble to bubble; Enter opens the panel
- **Result** (v0.4.0): [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

### MT-MSG-004: Start a conversation and answer, with a summary

- **Covers**: new conversation and reply forms (spec 008 US4, FR-006; spec 011 US5)
- **Prerequisites**: the two projects ready for messages
- **Steps**:
  1. Pick a project and press "New conversation" under its chat; choose the receiving project and the kind "question"
  2. Leave a required section empty and send; then fill every section, write a summary and send
  3. Write a summary longer than 280 characters in a reply and send
  4. Pick the other project, click the question and answer it with "answer" and a short summary
- **Expected**:
  - Step 1 offers only the project's counterparts and shows the Summary field (with a 0 / 280 counter) and the sections of the question template, the required ones marked
  - The first send in step 2 is refused next to the form with the missing sections named, and nothing is written; the second shows a sent bubble with the summary at once
  - Step 3: the counter turns red and the reply is refused with the summary named; nothing is written
  - Step 4: the answer appears as a sent bubble with its summary, quoting the question when it is not right above; after three rounds only agreement and impasse remain
- **Result** (v0.4.0): [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

### MT-MSG-005: News arrives without asking

- **Covers**: the messages worker and its interval (spec 010 US4, IMP-007, ADR-0012)
- **Prerequisites**: Aurora open on Messages, the interval at its default of 20 seconds (`messagesPollSeconds` in `settings.toml`)
- **Steps**:
  1. Open the Reply form of a conversation and type a few words; from a terminal in the other project's folder, send a question to this project
  2. Wait up to 30 seconds
  3. Minimize Aurora, send another message from the terminal, wait 30 seconds and bring Aurora back
  4. In `settings.toml`, set `messagesPollSeconds = 0` (off) and save; send another message from the terminal and wait two minutes
- **Expected**:
  - Step 2: the new conversation appears in the list by itself and the navigation count goes up; the Reply form keeps what was typed
  - Step 2 repeated with Portfolio open: the count next to Messages goes up there too
  - Step 3: the message appears within a few seconds of the window coming back (minimized, Aurora checks once a minute)
  - Step 4 shows no news until "Check now"
- **Result** (v0.3.0-rc.6): [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

### MT-MSG-006: The provider from a terminal

- **Covers**: `dcp-integrations` command line (spec 008 US6, FR-008, FR-009, FR-011)
- **Prerequisites**: the installed package (the provider sits next to `aurora`), a clone of a project's repository (with its `origin` remote) without `.aurora/integrations.json`
- **Steps**:
  1. Run `dcp-integrations version`, then `dcp-integrations check` in the folder
  2. Run `dcp-integrations init --project <slug>`, `dcp-integrations add <other-slug> --url <repository>` and `dcp-integrations validate`
  3. Run `dcp-integrations check --json`, then `dcp-integrations poll`; echo the exit code
- **Expected**:
  - Step 1 prints the version (Aurora's), then says `.aurora/integrations.json` is not found and gives the `init` command
  - Step 2 writes `.aurora/integrations.json` with the counterpart and `validate` reports it valid
  - Step 3 answers a document with `schema` `dcp-integrations.check` and `schema_version` 1.1; `poll` exits with 10 when there is news and 0 otherwise
- **Result** (v0.3.0-rc.1): [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

### MT-MSG-007: Without the provider or with a project not set up

- **Covers**: capabilities and setup states (spec 008 FR-011, FR-014), the setup noticed without reopening (IMP-004), the slug the network uses (BUG-009)
- **Prerequisites**: one registered project without `.aurora/integrations.json`; a second run with `dcp-integrations` removed from Aurora's folder and from the PATH
- **Steps**:
  1. Open Messages
  2. In a terminal, run the command shown for the project, then come back to Aurora's window
  3. Remove the provider, restart Aurora and open Messages
- **Expected**:
  - Step 1 lists the project as not set up with a command that runs from any folder: the full path of the provider, `--dir` with the project's folder and `init --project` with the slug the other projects' `.aurora/integrations.json` give to its repository (for a clone of ai-code-agent-skills, `dev-cycle-protocol`), or a slug from its name when none lists it; the other projects work normally
  - Step 2: within a few seconds of the window getting the focus, Messages offers to read again (or the count changes on another view); after reading again the project takes part, without restarting Aurora or reopening the view
  - Step 3 says once that the provider is missing and how to install it; nothing else runs and Aurora stays responsive
- **Result** (v0.3.0-rc.1): [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

### MT-MSG-008: Messages open fast and read conversations stop counting as new

- **Covers**: reads from the local copies, background sync, mark as read (IMP-005, IMP-006, ADR-0011)
- **Prerequisites**: several registered projects set up for messages, with a few dozen messages between them; Aurora's log open (`logs` under Aurora's data folder)
- **Steps**:
  1. Open Messages, leave it, wait a minute and open it again
  2. In a terminal, run `dcp-integrations sync` and then `dcp-integrations check --cached` in a project's folder
  3. From another project's folder, send a question to this project; in Aurora, press "Check now"
  4. Open a conversation with an unread dot
  5. Close Aurora, reopen it and open Messages
- **Expected**:
  - Step 1: the second opening lists the conversations within about a second; the log shows `check` with a small `elapsed_ms` and a `messages sync` line after it
  - Step 2: `sync` reports the inboxes it synchronized; `check --cached` answers at once
  - Step 3: the new question is listed after "Check now", without reopening the view
  - Step 4: the unread dot and the navigation count drop at once, without waiting; the conversation stays usable while it is marked
  - Step 5: the conversation opened in step 4 no longer shows the unread dot
- **Result** (v0.3.0-rc.1): [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

### MT-MSG-009: Messages open from the last snapshot, even offline

- **Covers**: the saved snapshot and the write queue (spec 010 US4, FR-005 to FR-007, ADR-0012)
- **Prerequisites**: Messages read at least once; a way to cut the network (airplane mode, or unplug)
- **Steps**:
  1. Close Aurora, cut the network, open Aurora and go to Messages
  2. Open a conversation with unread messages
  3. Bring the network back and wait 30 seconds
  4. Send a reply, then at once open another conversation
- **Expected**:
  - Step 1 shows the projects and the chat at once from the last snapshot; a project that cannot be reached says so in a short notice above its chat, and its last known messages stay listed
  - Step 2 marks the message read at once; Aurora stays responsive
  - Step 3: the notice goes away and the list is current, without reopening the view
  - Step 4: the reply appears as a sent bubble as soon as it is written, with no reload of the page; the other message opens without waiting for the write
- **Result** (v0.3.0-rc.6): [ ] Pass [ ] Fail [ ] Blocked
- **Result** (v0.4.0): [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

### MT-MSG-010: A summary sent from a terminal

- **Covers**: `--summary` of the provider (spec 011 US5, FR-001 to FR-003)
- **Prerequisites**: the installed package; two projects set up for messages
- **Steps**:
  1. In one project's folder, run `dcp-integrations send --to <other> --kind notice --subject "Test" --summary "I changed the export format; nothing to do on your side." --body-file <file>`
  2. Run the same with a summary of 300 characters
  3. In the other project's folder, run `dcp-integrations show <id> --json`; look at the message file on the `aurora/inbox` branch
  4. Open Aurora's Messages on the receiving project
- **Expected**:
  - Step 1 sends the message; step 2 is refused with exit code 5 and "at most 280"; nothing is written
  - Step 3: the header has `"summary"` with the text; the file has a `summary:` line after the other header fields and the body is unchanged
  - Step 4: the bubble shows the summary, not the subject
- **Result** (v0.4.0): [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

### MT-MSG-011: A project's logo

- **Covers**: project picture (spec 011 US1, FR-006)
- **Prerequisites**: a PNG or SVG under 256 KB, and an image over 256 KB
- **Steps**:
  1. In Portfolio, press "Logo…" on a project and pick the small image
  2. Open Messages
  3. Press "Logo…" again and pick the large image; then press "Remove logo"
- **Expected**:
  - Step 1: the project's card shows the image; nothing changes in the repository (`git status` is clean)
  - Step 2: the project's row, its chat header and the small pictures next to its received bubbles show the image
  - Step 3: the large image is refused with the 256 KB limit named; after "Remove logo" the initials come back
- **Result** (v0.4.0): [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

## Packages

These tests use the packages of a release (or of a dry run of the release workflow, in the run's artifacts), not a build from source.

### MT-PKG-001: Install, open and uninstall on Windows

- **Covers**: Windows installer (spec 006 US1, SC-002, SC-003)
- **Prerequisites**: Windows 10 22H2 or later, or Windows 11, without Aurora installed; `Aurora_<version>_x64-setup.exe` of the release
- **Steps**:
  1. Run the installer as a standard user (choose **More info**, **Run anyway** if SmartScreen asks) and accept the license
  2. Open Aurora from the Start menu, add a project and close Aurora
  3. Look at the size of Aurora's installation folder (`%LOCALAPPDATA%\Aurora`)
  4. Open a new terminal and run `dcp-integrations version`
  5. Uninstall Aurora from **Settings > Apps > Installed apps**, open a new terminal and run `dcp-integrations version` again; then install Aurora again and open it
- **Expected**:
  - Step 1: no administrator prompt; the installer shows the Levix Digital license and finishes
  - Step 2: the window opens; the sidebar shows the release's version
  - Step 3: under 30 MB
  - Step 4: prints the release's version (the installation folder is on the user's `PATH`)
  - Step 5: the Start menu entry is removed and the command is no longer found (the folder left `PATH`, other entries unchanged); after installing again, the project added in step 2 is still listed
- **Result** (v0.3.0-rc.1): [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

### MT-PKG-002: A newer installer updates an older installation

- **Covers**: update in place (spec 006 US1 scenario 5)
- **Prerequisites**: Aurora of an earlier release installed with its installer, with at least one project; the installer of the release under test
- **Steps**:
  1. Run the newer installer
  2. Open Aurora from the Start menu
- **Expected**:
  - Step 1: it installs over the earlier version without asking to uninstall it first
  - Step 2: the sidebar shows the new version and the project list is unchanged; **Settings > Apps** lists one Aurora
- **Result** (v0.3.0-rc.1): [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

### MT-PKG-003: Install, open and remove the .deb

- **Covers**: `.deb` package (spec 006 US2, SC-002, SC-003)
- **Prerequisites**: Ubuntu 22.04 or later without WebKitGTK 4.1 runtime installed if possible (a fresh virtual machine); the `.deb` of the release
- **Steps**:
  1. `sudo apt install ./<the .deb>`
  2. Open Aurora from the applications menu, add a project and close it
  3. `dpkg-query -W -f='${Installed-Size}\n' aurora`
  4. `sudo apt remove aurora`, then install the package again and open Aurora
- **Expected**:
  - Step 1: apt installs WebKitGTK 4.1 and GTK 3 if missing, and Aurora
  - Step 2: the menu entry has Aurora's icon; the window opens and the sidebar shows the release's version
  - Step 3: under 30720 (KiB)
  - Step 4: the remove deletes the menu entry and `/usr/bin/aurora`; after installing again, the project added in step 2 is still listed
- **Result** (v0.3.0-rc.1): [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

### MT-PKG-004: The AppImage runs

- **Covers**: AppImage package (spec 006 US2 scenario 2)
- **Prerequisites**: a Linux x86_64 desktop with FUSE 2 (Ubuntu 22.04 or later); the AppImage of the release
- **Steps**:
  1. `chmod +x <the AppImage>` and run it
- **Expected**:
  - The window opens with the release's version on the sidebar, without installing anything else
- **Result** (v0.3.0-rc.1): [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

### MT-PKG-005: The release lists the packages and their checksums

- **Covers**: release workflow (spec 006 US3, FR-005, SC-001)
- **Prerequisites**: the release under test
- **Steps**:
  1. Open the release's page and download every asset
  2. `sha256sum -c SHA256SUMS.txt` in the download folder
- **Expected**:
  - Step 1: a release candidate is marked "Pre-release" and not "Latest"; the assets are the Windows installer, the `.deb`, the AppImage and `SHA256SUMS.txt`
  - Step 2: every package reports `OK`
- **Result** (v0.3.0-rc.1): [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:
