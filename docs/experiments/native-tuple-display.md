# Native tuple Show and Debug

The Rust target constructs the compiler-selected `std/tuple::Show` and
`std/tuple::Debug` dictionaries from their ordered child evidence. Tuple
construction and pure entry inspection are separate operations: inspection
must not replace a source program's selected methods.

The emitter checks the structural `CoreType::Tuple`, standard trait identity,
element count, child constraints and dispatch method. User children retain
their canonical instance and provider owner. Their type arguments and
dependencies are checked against the instance's defining module. Parameter
children must match the enclosing function or instance constraint. Nested
standard children continue through their existing checked dictionary paths.

The runtime captures the selected child methods as ordinary managed values.
It invokes them from left to right, completing each call before the next. A
Defect stops the traversal. The tuple itself remains owned during callbacks,
and callbacks run without borrowing global, Ref or collector storage.
Captured dictionaries and closures remain visible to cycle collection.

Tuple and List use the same 128-level standard display bound as TypeScript.
Depth is restored on success and unwind. A user method keeps its own recursion
policy and may call another selected dictionary. Primitive rendering follows
the language's Show/Debug rules, including String escaping, Unicode Char,
non-finite Float and negative zero. This adds no Float Eq, Ord or Hash instance.

A closed tuple display dictionary may be retained by a partial call to a known
pure source function when all dictionaries are already selected and the
remaining source parameters have fixed primitive types. The source arity,
remaining arrow types and evidence staging are checked. Unproved deferred or
higher-kinded staging still fails before generating output.

The canonical fixtures cover custom, generic and higher-kinded children,
retained closures, partial calls, interpolation, nested Tuple/List and a
three-module path-package project. Effect fixtures cover reusable cold plans,
Ref callback cycles, typed tuple failure, finalizers and child Defects. Native
runtime tests and the Bun/Node probe also check shared depth, child order,
unwind restoration and dictionary/Ref cycle reclamation. These are behavioral
checks; this slice makes no performance improvement claim.
