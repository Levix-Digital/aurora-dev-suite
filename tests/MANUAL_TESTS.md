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
- **Result**: [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

### MT-WIN-002: Every view is reachable with the keyboard only

- **Covers**: navigation, focus styles (FR-002, SC-005)
- **Prerequisites**: Aurora open
- **Steps**:
  1. Press Tab until the focus is in the navigation; use the Down and Up arrows to move between the items and Enter to open each one
  2. Press Alt+1 to Alt+5
  3. In each view, press Tab through every control
- **Expected**:
  - The focused element always has a visible outline
  - Alt+1 to Alt+5 open Portfolio, Project, Engines, Inbox and Settings
  - Every button and field can be reached and used without the mouse
- **Result**: [ ] Pass [ ] Fail [ ] Blocked
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
- **Result**: [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

### MT-WIN-004: Project and Inbox say what they will show

- **Covers**: Project and Inbox views (FR-004)
- **Prerequisites**: Aurora open
- **Steps**:
  1. Open Project, then Inbox
- **Expected**:
  - Each says what it will show, "Coming in v0.2.0", and what the Dev Cycle Protocol must provide
- **Result**: [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

### MT-WIN-005: Linux without WebKitGTK

- **Covers**: start-up on Linux (edge case)
- **Prerequisites**: a Linux machine or container without `libwebkit2gtk-4.1-0`
- **Steps**:
  1. Run `aurora` from a terminal
- **Expected**:
  - The terminal names the missing library (`libwebkit2gtk-4.1.so.0`); the README's troubleshooting section tells which package to install
- **Result**: [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

### MT-WIN-006: The log file is written

- **Covers**: observability (plan)
- **Prerequisites**: Aurora started once
- **Steps**:
  1. Open the log folder: `~/.local/share/aurora/logs` on Linux, `%LOCALAPPDATA%\Levix\Aurora\data\logs` on Windows
  2. Start Aurora with `AURORA_LOG=debug` and open the Engines view
- **Expected**:
  - A file `aurora.<date>.log` has a "starting Aurora" line and one "engine probed" or "engine missing" line per engine; with `debug`, each engine command and its duration; never the engine's output
- **Result**: [ ] Pass [ ] Fail [ ] Blocked
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
- **Result**: [ ] Pass [ ] Fail [ ] Blocked
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
- **Result**: [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

### MT-PRJ-003: A moved project is shown as missing and can be located

- **Covers**: Portfolio, `relocate_project` (FR-009)
- **Prerequisites**: one registered repository
- **Steps**:
  1. Close Aurora, move the repository's folder, start Aurora
  2. Click "Locate…", type the new path and Save
- **Expected**:
  - Step 1 shows the project as "Missing" with its old path; step 2 shows it as "Present" at the new path
- **Result**: [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

### MT-PRJ-004: Removing a project never touches the repository

- **Covers**: Portfolio, `remove_project` (FR-006)
- **Prerequisites**: one registered repository
- **Steps**:
  1. Click Remove, then confirm
- **Expected**:
  - The project leaves the list; the repository folder is unchanged on disk
- **Result**: [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

### MT-PRJ-005: A damaged project list is kept aside

- **Covers**: registry recovery (FR-010)
- **Prerequisites**: Aurora closed; `projects.toml` in the configuration folder (`~/.config/aurora` on Linux, `%APPDATA%\Levix\Aurora\config` on Windows)
- **Steps**:
  1. Replace the content of `projects.toml` with `not [valid`
  2. Start Aurora
- **Expected**:
  - The Portfolio is empty and a notice gives the path of `projects.toml.damaged-<timestamp>`, which holds the old content
- **Result**: [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

## Foundations: engines

### MT-ENG-001: No engine installed

- **Covers**: Engines view (US3 scenario 1)
- **Prerequisites**: none of `dcp`, `gedo`, `actx` on the search path
- **Steps**:
  1. Open Engines
- **Expected**:
  - The three engines are "Missing" with what to install; every feature says what it needs; Aurora keeps working
- **Result**: [ ] Pass [ ] Fail [ ] Blocked
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
- **Result**: [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

### MT-ENG-003: A hanging engine only delays itself

- **Covers**: parallel probes, time limit (FR-015, SC-003)
- **Prerequisites**: fake engines installed as `gedo` (mode `hang`) and `actx` (mode `ready`); Gedo's time limit set to 3 seconds in Settings
- **Steps**:
  1. Click Refresh on Engines
- **Expected**:
  - AnyContext updates at once; Gedo turns "Degraded" ("no answer after 3 s") after about 3 seconds; "Last error output" opens
- **Result**: [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

### MT-ENG-004: A configured location wins over the search path

- **Covers**: Settings, probe (FR-011)
- **Prerequisites**: a fake engine copied as `gedo` into a folder that is not on the search path
- **Steps**:
  1. In Settings, set the Gedo location to that file and Save; refresh Engines
- **Expected**:
  - Gedo is "Ready" and its location is the configured file
- **Result**: [ ] Pass [ ] Fail [ ] Blocked
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
- **Result**: [ ] Pass [ ] Fail [ ] Blocked
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
- **Result**: [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

### MT-POR-002: Without dcp the Portfolio says what to install once

- **Covers**: unavailable state (spec 003 US3, SC-004)
- **Prerequisites**: no `dcp` on the search path and no `dcp` location in Settings; at least two projects registered
- **Steps**:
  1. Start Aurora and look at the Portfolio
- **Expected**:
  - The Portfolio line at the top says to install the Dev Cycle Protocol with its JSON contract
  - Every present project says "Cycle state unavailable." within a second; missing projects say to locate the folder
- **Result**: [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

### MT-POR-003: With dcp answering the contract, every project shows its summary

- **Covers**: Portfolio over the dcp adapter (spec 003 US1)
- **Prerequisites**: the fake engine set as the `dcp` location in Settings (it answers the contract by default); at least two present projects registered
- **Steps**:
  1. Refresh the engines, then open the Portfolio and click Refresh
- **Expected**:
  - Every present project shows "Implementation", "v0.4.0 Command line" and "3 wait for you"
  - While reading, the rows say "Reading the cycle state…" or "Refreshing…" and the header counts the reads
- **Result**: [ ] Pass [ ] Fail [ ] Blocked
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
- **Result**: [ ] Pass [ ] Fail [ ] Blocked
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
- **Result**: [ ] Pass [ ] Fail [ ] Blocked
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
- **Result**: [ ] Pass [ ] Fail [ ] Blocked
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
- **Result**: [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:

### MT-PVW-004: Without dcp the Project view says what to install once

- **Covers**: unavailable state (spec 004 US3, SC-003)
- **Prerequisites**: no `dcp` on the search path and no `dcp` location in Settings; at least one present project registered
- **Steps**:
  1. Start Aurora, press **Open** on a present project
- **Expected**:
  - The Project view line at the top says to install the Dev Cycle Protocol with its JSON contract
  - The view says "Cycle state unavailable." within a second
- **Result**: [ ] Pass [ ] Fail [ ] Blocked
- **Notes**:
