This ordinary two-module path package passes primitive JSON dictionaries across
the package boundary. It stores an exact Json behind an opaque Cell, preserves a
saved generic decoder and an encoder closure, and runs through strict raw
TypeScript, Bun, Node, and native Rust with no live managed roots at exit.
