import type {
  ExtensionContext,
} from 'vscode';
import {
  workspace,
} from 'vscode';
import type {
  LanguageClientOptions,
  ServerOptions,
} from 'vscode-languageclient/node';
import {
  LanguageClient,
  RevealOutputChannelOn,
  TransportKind,
} from 'vscode-languageclient/node';
import {
  ExtensionContextManager,
  LogManager,
} from './managers';

let client: LanguageClient | undefined;

export function activate (context: ExtensionContext) {
  ExtensionContextManager.initialize(context);

  const serverOptions: ServerOptions = {
    command: 'typedown-lsp',
    transport: TransportKind.stdio,
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [
      {
        scheme: 'file',
        language: 'typedown',
      },
    ],
    workspaceFolder: workspace.workspaceFolders?.[0],
    outputChannel: LogManager.getInstance().mainChannel,
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
  context.subscriptions.push(LogManager.getInstance());
}

export function deactivate (): Thenable<void> | undefined {
  return client?.stop();
}
