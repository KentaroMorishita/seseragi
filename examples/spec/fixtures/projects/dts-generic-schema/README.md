# Controlled generic schema fixture

`schema-api` is a local test host, not a published npm package. Its declaration
and JavaScript implementation exercise generic opaque binders, linked input and
return types, first-class functions, partial application, and checked return
codecs through DTS conversion and normal CLI builds on Node and Bun.

Registry package compatibility is covered separately by
`examples/spec/fixtures/semantic-interop-schema-1/npm` and the pinned
`base64-js@1.5.1` application fixture under Playground tests.
