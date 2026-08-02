const vscode = require("vscode")
const {
  CloseAction,
  ErrorAction,
  LanguageClient,
  RevealOutputChannelOn,
  State,
  TransportKind,
} = require("vscode-languageclient/node")
const { createExtensionController } = require("./extension-core")

const controller = createExtensionController({
  vscode,
  LanguageClient,
  TransportKind,
  State,
  RevealOutputChannelOn,
  ErrorAction,
  CloseAction,
})

module.exports = controller
