# Change Log

## 0.4.0

- Uses the shared Seseragi toolchain version and the unified `v<version>`
  release tag.
- Rejects a bundled or configured LSP whose toolchain version differs from the
  extension manifest.
- Verifies the native LSP's platform triple and executable permission before
  startup, and archive/extraction-smoke-tests every platform VSIX.
- Moves the official extension ID to `seseragi-dev.seseragi`; the former ID is
  published only as a non-LSP migration stub.

## 0.3.0

- Recognizes every `.ssrg` file as Seseragi, including untitled documents.
- Bundles a platform-specific `seseragi-lsp`; a custom path is now only an override.
- Adds language-server status, restart, output, version, and compatibility diagnostics.
- Packages and verifies macOS arm64/x64, Linux x64, and Windows x64 VSIX artifacts.

## 0.2.0

- Connected the TextMate grammar to the native Seseragi language server.

## 0.1.0

- Published the syntax-only Seseragi Spec Preview extension.
