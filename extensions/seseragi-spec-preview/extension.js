const vscode = require("vscode");
const {
  LanguageClient,
  TransportKind,
} = require("vscode-languageclient/node");

let client;

async function activate(context) {
  const command = vscode.workspace
    .getConfiguration("seseragi")
    .get("languageServer.path", "seseragi-lsp");
  const serverOptions = {
    command,
    args: [],
    transport: TransportKind.stdio,
  };
  const clientOptions = {
    documentSelector: [
      { scheme: "file", language: "seseragi" },
      { scheme: "untitled", language: "seseragi" },
    ],
    synchronize: {
      fileEvents: vscode.workspace.createFileSystemWatcher("**/*.ssrg"),
    },
  };

  client = new LanguageClient(
    "seseragi",
    "Seseragi Language Server",
    serverOptions,
    clientOptions,
  );
  context.subscriptions.push(client);
  await client.start();
}

async function deactivate() {
  if (client) {
    await client.stop();
  }
}

module.exports = { activate, deactivate };
