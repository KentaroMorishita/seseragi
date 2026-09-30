# Reader-facing reference acceptance

Route coverage, bilingual module presence, compilation and browser checks are
technical evidence. None of them proves that a person can understand an article.
The previous core-corpus completion claim is withdrawn for reader acceptance.

## What the reader must be able to do

An article must answer one named question without sending its reader away to
discover what that question means. It is not a Tour lesson, but being a reference
does not excuse missing explanations.

1. Explain the purpose in ordinary language before stating formal rules.
2. Introduce each necessary term where first used. Identify any genuine
   prerequisites, and summarize the part needed here; a prerequisite link alone
   is not an explanation.
3. Define every name in the first example. Start with the smallest example that
   demonstrates this concept, not an unrelated complete Tour lesson.
4. Explain how to read that example, what its important lines do, and why its
   output or type follows. Execute runnable examples and check actual output.
5. Distinguish the simple case from advanced rules. Explain type, evaluation,
   failure, resource and cost rules where they apply, with their consequences.
6. Describe a plausible mistake, its diagnostic or result, and how to correct it.
   Define additional names in rejected snippets or explain their relevance.
7. Avoid compiler-internal vocabulary unless the reader needs it. When needed,
   explain it; do not swap English jargon for equally unexplained Japanese jargon.
8. Review Japanese as an explanation written for a reader, not as a literal
   translation. Page title, breadcrumb, navigation labels and links must agree.
9. A related link states the question it helps answer. It is optional after the
   local explanation, not a mandatory unexplained jump.
10. Previous/next stays within the same navigation section. Show the current
    section and its overview. Category changes require a deliberate choice.

## Authoring boundary

Keep `page.ssrg`, `en.ssrg` and `ja.ssrg` as typed Seseragi. The existing semantic
Block model remains the sole article renderer. `reader-article.ssrg` shares a
reader-facing explanation pattern with arrays of page-owned paragraphs; it is
not a second document format. More specialised topics may use their own typed
sections rather than forcing irrelevant headings into this pattern.

Preserve normative traceability in the internal content map. Neither a public
specification-chapter label nor a link to a specification replaces explanation.
Do not silently change a specification or compiler behavior to make an example
appear to work. Record discrepancies and use genuinely executable examples.

## Review and completion

Review both rendered locales from the perspective of someone who knows basic
programming but not Seseragi's compiler implementation. Answer the ten questions
above for the page, not for the overall language. Read the text in sequence,
follow a necessary link and return, and check mobile/desktop layouts.

No minimum word count, heading count or automatic jargon detector establishes
reader acceptance. Automated checks protect examples, paragraph pairing, exact
titles, navigation boundaries and rendering. They cannot replace reading.

`reader-review.md` inventories all 106 core routes. Unchecked entries are not
accepted, including pages rewritten previously. Non-core semantic explanations
remain separate from generated API signature coverage in the existing corpus
backlog. Do not close the core corpus from route presence again.
