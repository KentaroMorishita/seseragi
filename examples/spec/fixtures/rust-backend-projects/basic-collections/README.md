# Basic native collections package

This ordinary two-module path dependency exercises generic Set creation,
partially applied updates, saved readers and immutable snapshots. Generic Map
membership and entry enumeration carry a saved callback across the package
boundary. The CLI integration test checks the original generated TypeScript
with strict settings and executes both profiles on Bun and Node alongside the
native binary, all with an empty runtime PATH.
