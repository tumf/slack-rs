---
description: Triage tumf/slack-rs GitHub issues (classify, respond, propose, and commit)
---

Triage open GitHub issues for `tumf/slack-rs`.

Goals:
- Classify each open issue.
- Reply to questions (only if related to slack-rs).
- For bugs/improvements: decide to proceed or decline.
- If declining: comment with rationale and close.
- If proceeding: create a proposal using skill `cflx-proposal`, and commit the proposal.

CRITICAL: GitHub comments are public. Do NOT include personal, secret, or confidential information.

**Input**

- Optional arguments: labels to focus, a limit (e.g. `/gh-issue-triage --limit 10 --label bug`).
- If no args: triage all open issues (reasonable default limit: 50).

**Steps**

1. **Collect open issues**

   Use `gh` to list open issues with enough fields to triage quickly.

   Example:
   ```bash
   gh issue list -R tumf/slack-rs --state open --limit 50 \
     --json number,title,author,labels,createdAt,updatedAt,url
   ```

2. **For each issue: fetch full details**

   ```bash
   gh issue view -R tumf/slack-rs <NUMBER> \
     --json number,title,body,author,labels,state,url,comments
   ```

3. **Classify**

   Put each issue into exactly one bucket:
- `question`: user is asking how to use slack-rs / behavior questions
- `bug`: incorrect behavior, crash, regression, broken docs
- `improvement`: feature request, enhancement, UX improvements
- `out_of_scope`: unrelated to slack-rs (ignore)

4. **Respond according to bucket**

   4.1 **out_of_scope**
   - Do nothing.

   4.2 **question**
   - Reply with a concise, actionable answer.
   - If it requires repo changes, treat as `improvement` instead.

   4.3 **bug / improvement**
   - Decide: `proceed` or `decline`.
   - Prefer `decline` when:
     - Not reproducible / insufficient info and the reporter is non-responsive
     - Conflicts with project goals / security posture / complexity too high
     - Requires secrets, private data, or proprietary context
   - Prefer `proceed` when:
     - Clear user value, aligned with slack-rs, implementable, safe

5. **Write GitHub comments (public-safe)**

   Use a consistent, non-sensitive style. Never include:
   - tokens, client secrets, signing keys
   - user emails, real names, workspace IDs, or private URLs
   - stack traces containing secrets

   5.1 **Answering questions**
   ```bash
   gh issue comment -R tumf/slack-rs <NUMBER> --body "<reply>"
   ```

   5.2 **Declining and closing**
   - Comment with rationale and (if possible) a safe alternative or workaround.
   - Close the issue.
   ```bash
   gh issue comment -R tumf/slack-rs <NUMBER> --body "<decline message>"
   gh issue close -R tumf/slack-rs <NUMBER> --comment "Closing as not planned."
   ```

6. **If proceeding: create a proposal and commit it**

   6.1 Load and run the proposal workflow:
   - Use the **Skill tool**: `cflx-proposal`
   - Create a proposal that references the GitHub issue (link + issue number), and includes:
     - problem statement
     - non-goals
     - approach options (at least 1)
     - acceptance criteria
     - risks / security considerations

   6.2 Commit the proposal artifacts
   - Stage ONLY the proposal-related files.
   - Use a message like:
     - `proposal: <short title> (#<issue>)`
   - Do not include secrets in commit messages.

7. **Comment back on the issue**

   After committing, comment with:
   - that a proposal has been created
   - where it lives (repo path)
   - what the next step is (e.g. review / implement)

   ```bash
   gh issue comment -R tumf/slack-rs <NUMBER> --body "Drafted a proposal at `<path>`; next step: review and start implementation."
   ```

**Output**

At the end, print a compact report:
- Total issues scanned
- Per-bucket counts
- For each issue touched: number + action (`answered`, `declined+closed`, `proposal+commit`) and links

**Guardrails**

- Only operate on `tumf/slack-rs`.
- Ignore anything unrelated to slack-rs.
- Never post or commit personal/secret/confidential information.
- Avoid long back-and-forth: if a bug report lacks required details, ask for the minimum reproducible info.
