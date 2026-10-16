# Native Array Show and Debug

The Rust target builds `std/array::Show` and `std/array::Debug` from the
compiler-selected element dictionary. Runtime inspection of an Array remains
separate from the language's selected Show/Debug methods.

Array and Tuple share the emitter's child evidence proof. An Array must have
the canonical prelude identity and exactly one element type and dictionary.
User and imported children retain their defining owner, instantiated type
arguments, and checked dependencies. Generic children must match an enclosing
dictionary parameter. Named aliases do not substitute for canonical identity.
The existing checked paths handle nested Array, Tuple, List and scalar children.

The runtime captures the selected element method as an ordinary managed edge.
It calls that method once per element, from left to right, and stops at a
Defect. An empty Array invokes no element method. Calls retain the immutable
Array without borrowing Ref, global or collector storage. Captured methods
participate in the existing Ref cycle collector and iterative graph teardown.

Array, Tuple and List use the shared 128-level standard display bound, restored
after success and Defect. String escapes, Unicode Char, non-finite Float and
negative zero follow the same rules as TypeScript. A closed Array display
dictionary can also be retained by a partial application with fixed remaining
primitive source parameters, using the same source-arity and staging checks as
Tuple. Unproved deferred evidence still fails before output generation.

Fixtures cover custom, generic and higher-kinded children, saved functions,
partial application, interpolation, mixed nested containers, module aliases
and path packages. Effect fixtures cover cold reuse, Ref callback cycles,
typed Array failures, finalizers and ordered child Defects. Runtime tests also
exercise shared depth, captured cycles and destruction of 100,000 nested
Arrays and display dictionaries on a 256 KiB stack.

This slice does not implement the remaining Array APIs or collection
typeclasses. It adds no standard Float Eq, Ord or Hash instance and makes no
performance improvement claim.

The shared frontend also retains resolved nominal owners in local instance-head
matching. Checked builtin types inferred for literals cannot be rebound to a
same-named local ADT in the semantic type catalog. The nominal Array fixture
mixes a user's Array and Int with literal arrays and integers; both targets
must select the corresponding user or standard methods.
