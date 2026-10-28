# CodeMirror selection mapping

`@codemirror%2Fstate@6.7.1.patch` collapses a selection after its contents are
replaced when mapping its endpoints would otherwise invert their order.
This corrects the actual selection, including the ranges stored by undo/redo;
the selection decorations keep their original validation and rendering.

Bun applies the tracked patch to both ESM and CommonJS through
`patchedDependencies` and the frozen lockfile. The published 6.7.6 package also
reproduces the invalid range, so a version update alone does not resolve #975.
Remove the patch when a compatible upstream release passes the replacement,
multiple-selection, and history cases in `tests/editor-editing.test.ts` and the
Format, Tour, and diagnostic navigation browser regressions.
