import type {
  ExtensionContext,
} from 'vscode';
import {
  RevealOutputChannelOn,
  window,
  workspace,
} from 'vscode';
import type {
  LanguageClientOptions,
  ServerOptions,
} from 'vscode-languageclient/node';
import {
  LanguageClient,
  TransportKind,
} from 'vscode-languageclient/node';

let client: LanguageClient | undefined;

export function activate (context: ExtensionContext) {
  const serverOptions: ServerOptions = {
    command: 'typedown-lsp',
    transport: TransportKind.stdio,
  };

  const outputChannel = window.createOutputChannel('Typedown LSP');

  const clientOptions: LanguageClientOptions = {
    documentSelector: [
      {
        scheme: 'file',
        language: 'typedown',
      },
    ],
    workspaceFolder: workspace.workspaceFolders?.[0],
    outputChannel,
    revealOutputChannelOn: RevealOutputChannelOn.Error,
  };

  client = new LanguageClient(
    'typedown-lsp',
    'Typedown LSP',
    serverOptions,
    clientOptions,
  );

  client.start();
  context.subscriptions.push(client);
}

export function deactivate (): Thenable<void> | undefined {
  return client?.stop();
}
