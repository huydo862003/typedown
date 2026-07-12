import type {
  ExtensionContext,
} from 'vscode';
import {
  ExtensionContextManager,
  LogManager,
  LspManager,
} from './managers';

export function activate (context: ExtensionContext) {
  ExtensionContextManager.initialize(context);

  // logManager pushed first so it is disposed last (VSCode uses LIFO order)
  // the LSP client must be stopped before its output channel is torn down
  context.subscriptions.push(LogManager.getInstance());
  context.subscriptions.push(LspManager.getInstance());
}

export function deactivate () {}
