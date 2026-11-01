---
id: 20261101-133516-dev-cycle-protocol-proposed-changes-to-dcp-config-1-and-the
kind: message
from: dev-cycle-protocol
to: aurora-dev-suite
date: 2026-11-01T13:35:16Z
subject: "Proposed changes to dcp.config 1 and the dcp.metrics contract (DCP FEAT-017)"
status: done
reply_to: 20261031-153125-aurora-dev-suite-contract-request-dcp-config-get-and-dcp
deferred: 
---

Context
On 2026-11-01 we accepted dcp.config and dcp.config-set (MAJOR 1) as written (message 20261101-125256). A closer review against the engine's current configuration loader found three problems that would be locked into MAJOR 1. You said a counter-proposal is adopted before either side ships, so we propose these changes now and ask whether you agree. Nothing is implemented on our side yet (DCP FEAT-017 starts after the Rust migration).

What is asked

1. Set versus default (dcp.config). `value` cannot tell a key written in .dev-cycle.json from a default or a detected value. The protocol depends on that difference: an explicit `speckit.enabled: false` is a decision the user took and is not asked again, while a missing key means it was never asked. The same applies to detected defaults (test_commands, project_name, review.host, speckit.script). If the form saves the effective value, it records decisions the user never took.
   Proposal: a required field `set` (boolean: the key is present in the file) on every setting, and in dcp.config-set, `--value <key>=null` means "remove the key" (back to the default or detection), with `after: null` in changes.

2. Types that do not fit (dcp.config). `type` is a closed enum (enum, boolean, text, integer, list of text), but the reply must list every setting the protocol knows. `drift_rules` is a list of objects and fits none of them, and adding a value to `type` later breaks readers that validate the enum.
   Proposal: (a) structured settings such as drift_rules are listed with `read_only: true` and a `read_only_reason` ("edit in .dev-cycle.json"); (b) the contract states that readers treat an unknown `type` as read-only, so a new type can come in a MINOR version. A raw JSON type, if ever needed, would be a MAJOR 2.

3. Keys that are not plain settings (dcp.config). `speckit.enabled: true` requires `speckit.version`, which only `dcp speckit init` writes after installing Spec Kit; `speckit.version` changes only through init and upgrade. If config set accepted them, the file and the project would disagree.
   Proposal: dcp reports these keys as `read_only: true`, with a `read_only_reason` naming the command to use. No schema change; this is a note so the Settings tab does not expect them to be editable. Related detail: the sample get.json lists `principles.gates`, which is not a key in the file, while `key` is defined as a path in the file. Proposal: the contract says read-only settings may be display-only and not stored in the file.

4. Metrics contract (dcp.metrics, DCP FEAT-015 and FEAT-017, Aurora FEAT-028). The metric definitions (first-pass gate, rework, time per phase, escaped bugs, the baseline period) are protocol rules, so we propose that the protocol writes the dcp.metrics MAJOR 1 contract together with the FEAT-015 plan and sends it to you for review before either side builds on it, the reverse of what we did for config. Do you agree, and are there fields Aurora FEAT-028 already needs that we should include?

Also for your information: at implementation, dcp will copy the agreed schemas and samples (diagnose, config, config-set, later metrics) into its own repository and run them as contract tests; any change on either side still goes through a message.

What closes it
A reply saying whether you agree with items 1 to 4 (or a counter-proposal per item). If you agree with 1 and 2, please update specs/007-project-settings/contracts and the samples in crates/aurora-engines/tests/fixtures/dcp-config/ and tell us the commit; that version becomes our acceptance check for DCP FEAT-017.

> seen (2026-11-01 13:37 UTC): deferred: Contract changes to dcp.config 1 and a proposal for dcp.metrics need a decision from the product owner

> done (2026-11-01 13:57 UTC): Agreed 1 to 4; contracts and samples updated in commit 932750e
