"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.deactivate = exports.activate = void 0;
const path = require("path");
const vscode_1 = require("vscode");
const node_1 = require("vscode-languageclient/node");
let client;
function activate(context) {
    // The server is implemented in node - look for it in the workspace root
    const workspaceRoot = vscode_1.workspace.workspaceFolders?.[0]?.uri.fsPath;
    if (!workspaceRoot) {
        console.error('No workspace folder found for Seseragi language server');
        return;
    }
    const serverModule = path.join(workspaceRoot, "dist", "lsp", "main.js");
    console.log('Seseragi LSP server path:', serverModule);
    // If the extension is launched in debug mode then the debug server options are used
    // Otherwise the run options are used
    const serverOptions = {
        run: { module: serverModule, transport: node_1.TransportKind.ipc },
        debug: {
            module: serverModule,
            transport: node_1.TransportKind.ipc,
        },
    };
    // Options to control the language client
    const clientOptions = {
        // Register the server for plain text documents
        documentSelector: [{ scheme: "file", language: "seseragi" }],
        synchronize: {
            // Notify the server about file changes to '.clientrc files contained in the workspace
            fileEvents: vscode_1.workspace.createFileSystemWatcher("**/.clientrc"),
        },
    };
    // Create the language client and start the client.
    client = new node_1.LanguageClient("seseragiLanguageServer", "Seseragi Language Server", serverOptions, clientOptions);
    // Start the client. This will also launch the server
    client.start().then(() => {
        console.log('Seseragi language server started successfully');
    }).catch((error) => {
        console.error('Failed to start Seseragi language server:', error);
    });
}
exports.activate = activate;
function deactivate() {
    if (!client) {
        return undefined;
    }
    return client.stop();
}
exports.deactivate = deactivate;
//# sourceMappingURL=extension.js.map