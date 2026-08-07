import type {
  Disposable, FileCreateEvent,
  CodeAction,
} from 'vscode';
import {
  commands, window, workspace, Position, Range, Selection,
} from 'vscode';

// Prompt users to select a schema when creating a new .td file
export class SchemaTemplateManager implements Disposable {
  private static instance: SchemaTemplateManager | undefined;
  private readonly subscription: Disposable;

  private constructor () {
    this.subscription = workspace.onDidCreateFiles((event) => {
      this.handleFilesCreated(event);
    });
  }

  static getInstance (): SchemaTemplateManager {
    if (!SchemaTemplateManager.instance) {
      SchemaTemplateManager.instance = new SchemaTemplateManager();
    }

    return SchemaTemplateManager.instance;
  }

  private async handleFilesCreated (event: FileCreateEvent) {
    for (const file of event.files) {
      if (!file.fsPath.endsWith('.td')) continue;

      // Wait briefly for the LSP to register the new file
      await new Promise((resolve) => setTimeout(resolve, 300));

      const document = await workspace.openTextDocument(file);
      const editor = await window.showTextDocument(document);

      // Only offer templates for empty or minimal files
      if (0 < document.getText().trim().length) continue;

      // Request code actions from the LSP
      const actions = await commands.executeCommand<CodeAction[]>(
        'vscode.executeCodeActionProvider',
        document.uri,
        new Range(new Position(0, 0), new Position(0, 0)),
      );

      if (!actions || actions.length === 0) continue;

      // Filter to "Initialize as ..." actions
      const schemaActions = actions.filter((action) => action.title.startsWith('Initialize as'));

      if (schemaActions.length === 0) continue;

      // Add an "Empty" option
      const items = [
        ...schemaActions.map((action) => ({
          label: action.title.replace('Initialize as ', ''),
          action,
        })),
        {
          label: '(No schema)',
          action: undefined,
        },
      ];

      const picked = await window.showQuickPick(items, {
        placeHolder: `Select schema for ${file.fsPath.split('/').pop()}`,
      });

      if (!picked || !picked.action) continue;

      // Apply the workspace edit from the code action
      if (picked.action.edit) {
        await workspace.applyEdit(picked.action.edit);
      }

      // Move cursor after the frontmatter
      const lastLine = document.lineCount - 1;

      editor.selection = new Selection(
        new Position(lastLine, 0),
        new Position(lastLine, 0),
      );
    }
  }

  dispose () {
    this.subscription.dispose();
  }
}
