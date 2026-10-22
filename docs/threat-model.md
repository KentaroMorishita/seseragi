# Playground execution threat model

## Assets and inputs

The editor owns workspace files, saved stdin, preferences, Tour progress and
browser navigation. Treat source links, edited Seseragi, imported declarations,
and foreign runtime JavaScript as untrusted. Opening a link starts compiler
analysis; execution requires Run or its keyboard shortcut.

## Execution boundaries

- The Rust compiler runs in a module Worker. A request deadline retires a hung
  Worker, rejects its queued requests and lets the next operation start a new one.
- A trusted, locally bundled bootstrap creates an opaque sandbox iframe without
  same-origin, popup or top-navigation permission. Its source/session handshake
  transfers a private MessagePort. Program bytes are sent after that handshake,
  outside the HTML parser. The evaluator must never enter editor chunks.
- Non-DOM programs, including foreign JavaScript, run in a dedicated classic
  Worker owned by that frame. A parent deadline cancels the frame and terminates
  its Worker; normal completion also terminates it. Arbitrary JavaScript may
  circumvent language-level checks, but does not gain editor DOM or storage.
- DOM programs run in the opaque frame with checkpoints in generated loops and
  function bodies. Foreign runtime JavaScript with DOM is refused because it
  could evade those checkpoints and block the editor event loop.
- Output is bounded before accumulation. Text is rendered with textContent;
  HTML output is rendered only when Preview is selected, with scripts denied.

## Data and network authority

The parent Storage broker checks the entry capability, operation, request ID,
settled state and pending-operation budget. It adds an editor-selected workspace
identity to every application key and enforces a per-area size budget. Programs
cannot choose that identity through the broker. Tour uses separate lesson IDs.
Legacy shared application data is preserved but not exposed by the new broker.
Editing code in the same workspace retains that workspace's storage authority;
storage isolation is not a per-source authorization system.

Source links start with empty stdin in a separate persistent workspace slot.
Only after a successful initial save is the source parameter replaced by the
slot identifier. Reload restores that slot's edits. The ordinary workspace key
is never overwritten by shared-slot edits or UI changes. Stdin is restored only
with its own saved workspace.

HTTP/WebSocket destinations and external HTTPS images are blocked by CSP until
the user enables Tools → Network. Permission is not persisted and resets when program
files/project/lesson change. Network-enabled code can send its input and its
workspace application data to external services. Browser restrictions still
apply. This is not a total network sandbox: ordinary same-frame link navigation
is possible. The opaque frame has no authority to navigate the editor itself.
Type-level capabilities and byte digests do not prove a package author is safe.

## Network provider boundaries

Programs using the official HTTP and WebSocket host providers also receive
untrusted peer data. The public HTTP `sendBytes` and `sendEmpty` helpers collect
the existing response stream and check the caller's byte limit before retaining
each chunk. Overflow returns `HttpResponseBodyLimitExceeded` and closes the
exchange without waiting for peer EOF. Browser response readers and pending
fetches are cancelled when their exchange closes. The internal one-shot `send`
operation remains available for compatibility; it has no caller-supplied body
limit and is not the implementation of these bounded public helpers.

The Node HTTP server bounds one incoming request body to 8 MiB by default,
checks both declared and received lengths, and returns 413 before dispatching
an oversized request to the application. The Bun server uses its native limit
(128 MiB in the pinned Bun 1.4.4). Interrupted uploads are released on server
close. These are per-request limits, not a process-wide or concurrency quota.

The native small-response helpers decode known content encodings incrementally,
with at most eight decoding stages. Their limit applies to decoded chunks,
including compressed input that expands beyond the limit. Native public
`exchange` preserves content-encoded body bytes for explicit decoders and proxy
forwarding; the browser provider returns Fetch-decoded bytes for both APIs.
Cancellation owns both the transport and any helper decoder cleanup. Use Bun
1.4.4 or a patched supported Node release: buffering limits do not repair a
vulnerable host Brotli decoder.

Bun small-response helpers preserve the Fetch proxy environment: scheme-specific
HTTP_PROXY/HTTPS_PROXY, ALL_PROXY fallback, lowercase precedence and NO_PROXY
host, port, IP and CIDR exclusions. HTTP forwarding and HTTPS CONNECT retain
the bounded response parser. CONNECT response headers and informational messages
are bounded before destination TLS starts. Proxy credentials stay on the proxy
leg, certificate verification applies to both TLS legs, and cancellation owns
pending and established sockets. Public native `exchange` keeps its existing
direct transport behavior.

Both native HTTP transports bound informational responses to 16 before a final
response, including Node responses queued while consumer demand is zero. The
Bun HTTP/1 transport also bounds response headers and trailers to 16 KiB each
and chunk-size lines to 1 KiB. Incomplete metadata is checked as bytes arrive.
Overflow and bytes remaining after a completed body produce a protocol failure and close the
socket. Ambiguous transfer-encoding/content-length combinations, invalid header
names and control characters, unsafe content lengths and out-of-range status
codes are rejected before publishing the final head. Backpressure may defer
parsing buffered bytes after EOF; it must not turn a complete response into a
premature-EOF error.

Node WebSocket receive errors fail and terminate the affected connection.
Invalid upgrade URLs are rejected before dispatch. Both Node and Bun observe
handler rejection and release connection resources; one failed handler must
not become an unhandled process-level rejection. These controls do not replace
application authentication, connection limits or a host process sandbox.

## Compiler input processing

The shared Rust syntax parser reads infix operands once before applying known
operator precedence. One synchronous parse shares a recursion and work budget
across helper parsers, including block, do, match and template ranges. Its
current limits are 64 recursive parser frames, expression depth 128,
cumulative pattern depth 512 and 1,000,000 parsing/checking steps.
Constructor pattern prefixes are scanned once and folded without a native
recursive frame per constructor, preserving the existing match-coverage limit.
Custom infix chains account for their later
curried application depth. Exceeding a limit reports `SES-P0002` with a UTF-8
source range; it must not overflow the host stack. Template scanning uses an
explicit stack before the parser budget applies.

Declaration conversion stores one overload set and appends each signature and
origin once, rather than retaining every previous prefix. These controls remove
the grouped-expression and overload-storage amplification paths. They do not
provide a total compiler CPU or heap quota; the browser compiler Worker deadline
remains a separate availability boundary.

## Availability limits and remaining assumptions

The compiler request deadline is 15 seconds; non-DOM execution is 10 seconds.
DOM checkpoints cap a synchronous generated-code turn by elapsed time and count.
These guards are not a replacement for a JS interpreter with execution fuel.
There is no browser API for a strict per-program heap quota. Native built-in
operations, huge allocations, browser bugs and process scheduling remain outside
the hard guarantees. Browser sandbox/CSP enforcement assumes a supported,
updated browser. No server-side program execution or privileged filesystem/
process bridge is introduced by the Playground.

## Review and verification

For PRs changing these boundaries, review the actual CSP, port consumer, broker,
Worker retirement and import/compile paths. Check GitHub code and security review
findings at the final head as well as CI. The automatic security review's high
severity threshold does not establish that medium/low issues are absent.
Use harmless local markers and intercepted network fixtures for browser checks;
do not use real secrets or external exfiltration endpoints.
