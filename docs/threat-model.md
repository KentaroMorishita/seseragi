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
