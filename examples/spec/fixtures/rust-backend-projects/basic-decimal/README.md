# Exact Decimal path package

Two ordinary modules pass an opaque Decimal and a saved formatter across a path
dependency. The fixture keeps a large fractional value exact, accepts arbitrary
exponents without expanding them, and rejects the first invalid Unicode byte.
The CLI test compares the original strict TypeScript in both profiles on Bun
and Node with the native binary, all with an empty runtime PATH.
