const vscode = require("vscode")

const OFFICIAL_EXTENSION_ID = "seseragi-dev.seseragi"
const LEGACY_STUB_KIND = "seseragi-legacy-migration-stub"

async function showMigration() {
  const action = await vscode.window.showWarningMessage(
    "The Seseragi extension moved to seseragi-dev.seseragi. Install the official Seseragi extension, then uninstall this legacy migration stub.",
    "Open Extensions"
  )
  if (action === "Open Extensions") {
    await vscode.commands.executeCommand(
      "workbench.extensions.search",
      `@id:${OFFICIAL_EXTENSION_ID}`
    )
  }
}

async function activate(context) {
  context.subscriptions.push(
    vscode.commands.registerCommand(
      "seseragiLegacy.openMigration",
      showMigration
    )
  )
  if (!vscode.extensions.getExtension(OFFICIAL_EXTENSION_ID)) {
    await showMigration()
  }
  return { kind: LEGACY_STUB_KIND }
}

module.exports = { LEGACY_STUB_KIND, activate }
