# Seseragi Legacy Migration

This package intentionally keeps the former Marketplace ID
`seseragi-dev.seseragi-spec-preview`. It is a migration stub: it never bundles
or starts `seseragi-lsp`.

## Move to the official extension

1. Install the platform-specific official VSIX, such as
   `seseragi-v0.4.0-vscode-darwin-arm64.vsix`.
2. Disable or uninstall this legacy extension after the official `Seseragi`
   extension appears in VS Code.
3. If your settings contain
   `"editor.defaultFormatter": "seseragi-dev.seseragi-spec-preview"` for
   `[seseragi]`, run **Seseragi: Migrate Legacy Settings** or replace it with
   `seseragi-dev.seseragi`.

The custom server setting remains
`seseragi.languageServer.path`; no value is discarded during this migration.
