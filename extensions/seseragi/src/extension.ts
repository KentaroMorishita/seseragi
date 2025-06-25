import * as path from "path";
import { workspace, ExtensionContext } from "vscode";

import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from "vscode-languageclient/node";

let client: LanguageClient;

export function activate(context: ExtensionContext) {
  // The server is implemented in node - look for it in the workspace root
  const workspaceRoot = workspace.workspaceFolders?.[0]?.uri.fsPath;
  if (!workspaceRoot) {
    console.error('No workspace folder found for Seseragi language server');
    return;
  }
  
  const serverModule = path.join(workspaceRoot, "dist", "lsp", "main.js");
  console.log('Seseragi LSP server path:', serverModule);

  // If the extension is launched in debug mode then the debug server options are used
  // Otherwise the run options are used
  const serverOptions: ServerOptions = {
    run: { module: serverModule, transport: TransportKind.ipc },
    debug: {
      module: serverModule,
      transport: TransportKind.ipc,
    },
  };

  // Options to control the language client
  const clientOptions: LanguageClientOptions = {
    // Register the server for plain text documents
    documentSelector: [{ scheme: "file", language: "seseragi" }],
    synchronize: {
      // Notify the server about file changes to '.clientrc files contained in the workspace
      fileEvents: workspace.createFileSystemWatcher("**/.clientrc"),
    },
  };

  // Create the language client and start the client.
  client = new LanguageClient(
    "seseragiLanguageServer",
    "Seseragi Language Server",
    serverOptions,
    clientOptions,
  );

  // Start the client. This will also launch the server
  client.start().then(() => {
    console.log('Seseragi language server started successfully');
  }).catch((error) => {
    console.error('Failed to start Seseragi language server:', error);
  });
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}