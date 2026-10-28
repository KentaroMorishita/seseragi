# Integer power file transform

The path dependency parses each input line as an exponent and computes `2 ** n`.
The main package writes the complete result to `output.txt` only after every
line succeeds. `native_power` CLI tests copy this package, refresh its ordinary
TypeScript provider lock with the current toolchain, and freeze all manifests,
sources and the lock before building either target.

The tests compare the original generated TypeScript in development and release
profiles under strict checking, fresh Bun and Node processes, and a native
executable with an empty PATH. Valid input includes exponents 0 and 52; negative
exponents and overflow are Defects, while malformed decimal and Unicode input
remain typed failures. Every failing run preserves the input and old output.
The lock continues to describe the shared TypeScript target rather than implying
that native execution selected a TypeScript provider.
