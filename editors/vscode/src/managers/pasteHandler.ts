import type {
  Disposable,
} from 'vscode';
import {
  DocumentDropOrPasteEditKind,
  languages,
} from 'vscode';
import {
  MIME_TO_EXTENSION,
  PasteEditProvider,
} from '../providers/pasteHandler';

export class PasteHandlerManager implements Disposable {
  private static instance: PasteHandlerManager | undefined;
  private readonly registration: Disposable;

  private constructor () {
    this.registration = languages.registerDocumentPasteEditProvider(
      {
        scheme: 'file',
        language: 'tdr',
      },
      new PasteEditProvider(),
      {
        providedPasteEditKinds: [DocumentDropOrPasteEditKind.Empty],
        pasteMimeTypes: [...MIME_TO_EXTENSION.keys()],
      },
    );
  }

  static getInstance (): PasteHandlerManager {
    if (!PasteHandlerManager.instance) {
      PasteHandlerManager.instance = new PasteHandlerManager();
    }

    return PasteHandlerManager.instance;
  }

  dispose (): void {
    this.registration.dispose();
    PasteHandlerManager.instance = undefined;
  }
}
