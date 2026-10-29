# Byte values across a path package

The root and `vendor/buffers` exchange canonical `Byte`, `Bytes`, and
`ByteError` values. A saved reader captures an immutable buffer, reads its
unsigned values, and returns `Nothing` beyond the end. Conversion rejects the
first invalid Int without exposing a partial buffer.

The CLI regression compares the original generated TypeScript in both build
profiles, fresh Bun and Node processes, and native execution with an empty PATH.
