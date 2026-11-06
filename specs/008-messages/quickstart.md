# Quickstart: Messages between projects

## The provider from a terminal

```bash
cargo build -p aurora-integrations --release
P=target/release/dcp-integrations
cd ~/src/aurora-dev-suite
$P init --project aurora-dev-suite
$P add dev-cycle-protocol --url https://github.com/Levix-Digital/ai-code-agent-skills.git --name "Dev Cycle Protocol"
$P validate                      # then commit .aurora/integrations.json
$P poll; echo $?                 # 0 nothing new, 10 news
$P check
$P template proposal > body.md   # fill it
$P send --to dev-cycle-protocol --kind proposal --subject "Try the provider" --body-file body.md
$P record <thread>               # once the other side closes the conversation
$P decide <thread> --choice "1"  # or: --revise "change X"; or: guide <thread> --text "..."
```

## The Messages view in the preview

```bash
cd crates/aurora-ui && trunk serve    # open http://localhost:1420, then "Messages"
```

1. The first card is the sample closing record "Release v0.6.0 after dcp v2.11.0" of `aurora-dev-suite` and `dev-cycle-protocol` (2 rounds): conclusion, justification, decision needed with options 1 and 2 (1 recommended), after approval.
2. Pick option 1 and confirm: the card leaves "Waiting for you" and the conversation shows "Decided: 1".
3. On the sample "Shared settings file" conversation, press "Ask for changes", write "Keep the plugin declarations too" and send: the conversation shows cycle 2 and the request.
4. Filter by `gedo` and open "Inbox format version": three messages in order with their sections; the last one is marked "no answer yet".
5. Press "New conversation", pick `aurora-dev-suite` to `gedo`, kind proposal, leave Proposal empty and send: the form names "Proposal"; fill it and send.
6. Wait for the simulated poll (10 seconds in the preview): "Messages" in the navigation shows 1.
7. `levix-admin-hub` is listed under "Not set up" with `dcp-integrations init --project levix-admin-hub`.
