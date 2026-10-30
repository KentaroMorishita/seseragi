Original two-module locked package for native Set.map. The provider owns opaque
input and output keys. Its generic mapping only requires output Eq/Hash, and
the caller saves the constrained imported function before supplying its mapper.
Tests retain both original raw TS profiles and compare Bun, Node and native.
