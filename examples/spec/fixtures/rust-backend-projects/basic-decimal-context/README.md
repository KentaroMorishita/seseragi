# Native DecimalContext across an opaque package boundary

The provider owns Context and reexports canonical RoundingMode and context-error
aliases. The caller saves generic/partial constructors and captures an opaque
value through a getter closure. Original raw TypeScript in both profiles is
checked with strict/skipLibCheck:false and compared in fresh Bun/Node/native.
