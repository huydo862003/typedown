import {
  basename,
  dirname,
  join,
  relative,
} from 'node:path';
import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import type {
  CancellationToken,
  DataTransfer,
  DataTransferItem,
  DocumentPasteEditContext,
  DocumentPasteEditProvider,
  Range,
  TextDocument,
} from 'vscode';
import {
  DocumentDropOrPasteEditKind,
  DocumentPasteEdit,
  SnippetString,
} from 'vscode';
import {
  LspManager,
} from '../managers/lsp';

// Keep in sync with AssetKind::from_extension in tdr-lang
export const MIME_TO_EXTENSION: ReadonlyMap<string, string> = new Map([
  [
    'image/png',
    '.png',
  ],
  [
    'image/jpeg',
    '.jpg',
  ],
  [
    'image/svg+xml',
    '.svg',
  ],
  [
    'image/webp',
    '.webp',
  ],
  [
    'application/pdf',
    '.pdf',
  ],
]);

export class PasteEditProvider implements DocumentPasteEditProvider {
  async provideDocumentPasteEdits (
    document: TextDocument,
    _ranges: readonly Range[],
    dataTransfer: DataTransfer,
    _context: DocumentPasteEditContext,
    _token: CancellationToken,
  ): Promise<DocumentPasteEdit[] | undefined> {
    const match = findSupportedMime(dataTransfer);

    if (!match) {
      return undefined;
    }

    const extension = MIME_TO_EXTENSION.get(match.mimeType)!;

    const [
      binaryData,
      assetsResponse,
    ] = await Promise.all([
      match.item.asFile()?.data(),
      LspManager.getAssetsDir(document.uri.toString())
        .catch((): AssetsDirectoryResponse => ({
          mode: 'local',
          path: 'assets',
        })),
    ]);

    if (!binaryData) {
      return undefined;
    }

    const documentDirectory = dirname(document.uri.fsPath);
    const assetsDirectory = join(documentDirectory, assetsResponse.path);

    await mkdir(assetsDirectory, {
      recursive: true,
    });

    const stem = basename(document.uri.fsPath).replace(/\.[^.]+$/, '') || 'untitled';
    const filename = `${stem}-${Date.now()}${extension}`;
    const filePath = join(assetsDirectory, filename);

    await writeFile(filePath, binaryData);

    const relativePath = relative(documentDirectory, filePath);
    const snippet = new SnippetString(`\${fref("${relativePath}")}`);

    return [new DocumentPasteEdit(snippet, 'Paste as TDR asset', DocumentDropOrPasteEditKind.Empty)];
  }
}

interface AssetsDirectoryResponse {
  mode: 'local';
  path: string;
}

function findSupportedMime (dataTransfer: DataTransfer): {
  mimeType: string;
  item: DataTransferItem;
} | undefined {
  for (const mimeType of MIME_TO_EXTENSION.keys()) {
    const item = dataTransfer.get(mimeType);

    if (item) {
      return {
        mimeType,
        item,
      };
    }
  }

  return undefined;
}
