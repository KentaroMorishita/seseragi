# VS Code product-test archive transport

Tracking: [#824](https://github.com/KentaroMorishita/seseragi/issues/824).

The product E2E runner used `@vscode/test-electron` under Bun 1.3.11.
The SDK pipes an HTTPS archive stream into `child_process.spawn("tar")` and
validates its byte length and SHA256. Linux CI repeatedly reported gzip/tar
Unexpected EOF and EPIPE before starting the installed-product journey.

## Isolation

The VS Code 1.133.0 Linux archive was 347,821,409 bytes, with SHA256
`2bf1a90d2f008af009eb3c4a7bd0849b9247a0588cd39a404a0b1e691be68161`.
Instrumented real-download controls on macOS received that complete body under
both Bun 1.3.11 and Node 24.14.0 and extracted it successfully. That success
alone did not reproduce the Linux failure.

Replaying the same verified archive through a file Readable into a slow byte
receiver removed the CDN and tar from the experiment. Bun submitted every byte,
but the receiver obtained only 347,799,552 bytes with 64 KiB chunks, and
347,078,656 bytes with 1 MiB chunks. Both cases reported EPIPE. Node preserved
the complete length and SHA256 for both chunk sizes. The original failed CI
download did not record byte counters, so these are controlled reproductions
of the SDK's transport path rather than measurements of that failed request.

The committed, network-free probe uses a smaller deterministic payload with a
partial final chunk. It records submitted/received byte counts and hashes:

```sh
bun scripts/probes/local-web-product-e2e-transport-probe.cjs target/pipe-bun.json
node scripts/probes/local-web-product-e2e-transport-probe.cjs target/pipe-node.json
```

Bun loss is recorded as diagnostic evidence. Node loss fails the control,
because the product SDK runner uses Node. These probes do not replace the
actual product E2E assertions.

## Product runner

The Bun orchestration script prepares the native archive, canonical Web starter
and private extension/user-data directories. A Node SDK runner downloads VS
Code, installs the supplied VSIX and launches the original Extension Host test.
CI pins Node 24.22.1 alongside the existing Bun toolchain. The Node pin includes
the October 21, 2026 security fixes; the measurements above used Node 24.14.0.

Each journey gets a fresh private SDK cache. The SDK's archive validation is
retained, installation cannot start after download rejection, and both failed
and completed caches are removed. `provisioning.json` distinguishes completed
download from completed product assertions; failures propagate to the caller.
CI retains provisioning and transport evidence even on failure.

The original browser, CLI, LSP, diagnostic recovery and source-map assertions
remain required. This changes the testing host for the VS Code SDK; it does not
add a Node dependency to emitted native programs or fix Bun itself.
