Original two-module locked package for native Map upsert, mergeWith and
mapKeysWith. The provider owns opaque input and output keys; generic rekey only
requires the output Eq/Hash dictionaries. Tests keep original raw TypeScript in
both profiles and compare fresh Bun, Node and native results and snapshots.
