reduceUntil uses an imported Iterable for opaque source and accumulator types,
a public ReduceStep alias and generic saved factories. Done preserves the
accumulator while skipping the remaining source; repeated calls replay input.
Both original strict TypeScript profiles and fresh Bun/Node/native use the frozen
path dependency. The standalone Done constructor remains a saved function value.
