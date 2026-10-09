# Docs reader review protocol

The sole editorial policy is [editorial-contract.md](editorial-contract.md),
established by Docs Reboot #601 / #763. This file owns the review protocol only.
Its former TypeScript-only audience, mandatory explanation sequence,
`reader-article` authoring prescription and #698–#701 work sequence are retired.
Earlier verification and reader-session files remain historical evidence;
they do not accept the new chapter or re-open the old work queue.

## Review a changed route or chapter

Record the exact commit, route and locale, the reviewer's programming background,
and whether this is agent self-review, author review or an actual first-time
reader session. Programming experience is required; no particular language,
Rust, Haskell or category theory is assumed.

Read each article both in chapter order and directly from its deep link.
Ask the reader to explain the task, important notation, input, result and how
they would change the example. Record where they needed help. Check that they
can find an exact rule and return, and choose a useful next destination.
Questions are review prompts, not compulsory public headings or prose quotas.
The applicable semantic obligations in `reference-content-map.md` remain;
their placement can change or move to a documented linked destination.

Review Japanese and English as natural writing with matching meaning, examples,
constraints and links. Japanese-first drafting is a workflow, not a reason to
translate sentence structure literally. Fair comparisons with another language
are optional; where used, verify both programs' input, output and restrictions.

## Keep evidence separate

| Evidence | Record |
| --- | --- |
| Code and meaning | Source path/hash, current CLI version/commit, command, stdout/diagnostic, formatter result, spec and compiler provenance |
| Display and navigation | Both locales, desktop/mobile, no-JS, keyboard/a11y, heading anchors, locale identity, related-link round trips and chapter navigation |
| Editorial self-review | Concrete observations against the editorial contract; unresolved style choices for the author |
| Author review | Actual decision about prose/code aesthetics; do not infer it from agent review |
| First-time reader | Background, observed understanding, help required, feedback, corrections and subsequent review |

No category substitutes for another. A blocked build or browser check stays
blocked. Automatic word/heading/paragraph counts cannot establish understanding.
#702 records the new chapter's reading evidence; #706 and #740 own independent
SSG/browser gates. #631 owns root cutover after the #601 prerequisites.

## Existing records

`reader-review.md` retains the previous 106-core-route review and API history.
Checked boxes there describe the review standard and commit recorded there;
they are not Docs Reboot acceptance. Preserve actual reader feedback and source
execution evidence and link them from each migration batch. Do not bulk-check
old rows, erase outstanding work, or treat published text as an approved draft.
Local `*-work-item.md` files are old scope/evidence records, not dispatch orders.

The earlier yomiyasu `tech` pass and pinned tool version are historical/advisory.
If reused, lint prose extracted from rendered Japanese pages, excluding code,
and record intentional technical exceptions. Lint results do not replace author
or reader review and do not impose a formal tone on the new conversational prose.
