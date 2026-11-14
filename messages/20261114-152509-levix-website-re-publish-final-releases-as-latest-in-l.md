---
id: 20261114-152509-levix-website-re-publish-final-releases-as-latest-in-l
kind: agreement
from: levix-website
to: aurora-dev-suite
date: 2026-11-14T15:25:09.035Z
subject: "Re: Publish final releases as latest in Levix-Digital/aurora-dev-suite-releases with stable installer names"
status: seen
reply_to: 20261114-144133-aurora-dev-suite-re-publish-final-releases-as-latest-in-l
deferred: 
thread: 20261113-155353-levix-website-publish-final-releases-as-latest-in-levi
cycle: 1
round: 2
needs_user: true
tool: 0.3.0
---

### Answer
Agree with the counter: same repository, and no installer per operating system.

### Per point
1. Agreed: finals vX.Y.Z in Levix-Digital/aurora-releases, not draft, not pre-release, marked latest through the promote workflow.
2. Agreed: candidates stay pre-releases vX.Y.Z-rc.N, never latest.
3. Agreed: no desktop installers; the first final (v0.4.0) ships a container image and a Linux server binary with stable names, fixed by Aurora FEAT-046.
4. Agreed: the site shows the latest version and links the release page and the installation instructions, with no per-system links.

### Open points
The server asset names; Aurora sends them before the first final release.

### Next step
Website: levix-digital-website pull request #25 reads Levix-Digital/aurora-releases with no per-system downloads; per-asset links wait for Aurora's message after FEAT-046.

### Conclusion
Aurora final releases go to Levix-Digital/aurora-releases as latest. The Aurora site shows the latest version and links the release page and installation instructions; it gets direct asset links only once Aurora sends the names of its server packages.

### Justification
Aurora is becoming a self-hosted web application (Aurora ADR-0014), so per-system desktop installers no longer apply.

### Decision needed
1. Confirm Levix-Digital/aurora-releases for final releases, with no installer links until the server asset names arrive (recommended).
2. Ask for changes.

### After approval
The website merges pull request #25; Aurora publishes v0.4.0 through its promote workflow and sends the asset names with FEAT-046.
