# Native Decimal conversions across an opaque package boundary

The provider owns Value and reexports the canonical standard conversion-error
alias. The caller keeps a captured opaque value and saved/generic getters.
Original raw TypeScript in both profiles is checked with strict and
skipLibCheck:false and compared in fresh Bun/Node/native processes.
