// Adapted from VitePress localSearchPlugin.ts
// https://github.com/vuejs/vitepress/blob/main/src/node/plugins/localSearchPlugin.ts
//
import MiniSearch from 'minisearch';
import {
  SEARCH_FIELDS, SEARCH_STORE_FIELDS, stripHtml,
} from '@/shared';

// Rendered page passed to the indexer
export interface PageIndexInput {
  id: string;
  title: string;
  html: string;
}

// Single section stored in the MiniSearch index
interface SearchIndexEntry {
  id: string;
  title: string;
  titles: string[];
  text: string;
}

// Matches heading tags that contain anchor links
const headingRegex = /<h(\d*).*?>(.*?<a.*? href="#.*?".*?>.*?<\/a>)<\/h\1>/gi;
// Extracts heading text and anchor id from the inner HTML
const headingContentRegex = /(.*)<a.*? href="#(.*?)".*?>.*?<\/a>/i;

export class SearchIndexer {
  private index: MiniSearch<SearchIndexEntry>;
  // Tracks which document ids belong to each page, for incremental updates
  private pageDocIds = new Map<string, string[]>();

  constructor () {
    this.index = new MiniSearch<SearchIndexEntry>({
      fields: SEARCH_FIELDS,
      storeFields: SEARCH_STORE_FIELDS,
    });
  }

  // Replace the entire index with a new set of pages
  addAll (pages: PageIndexInput[]): void {
    // Discard pages that are no longer present
    const incoming = new Set(pages.map((page) => page.id));

    for (const id of this.pageDocIds.keys()) {
      if (!incoming.has(id)) this.discardPage(id);
    }
    for (const page of pages) {
      this.addPage(page);
    }
  }

  // Index a single page by splitting its HTML into heading sections
  addPage (page: PageIndexInput): void {
    this.discardPage(page.id);

    const documentIds: string[] = [];

    // Each section becomes a separate entry keyed by page#anchor
    for (const section of splitPageIntoSections(page.html)) {
      const documentId = section.anchor ? `${page.id}#${section.anchor}` : page.id;

      if (this.index.has(documentId)) this.index.discard(documentId);
      this.index.add({
        id: documentId,
        text: section.text,
        // Last title in the hierarchy is the section heading
        title: section.titles.at(-1) ?? page.title,
        // Parent titles form the breadcrumb trail
        titles: 1 < section.titles.length
          ? section.titles.slice(0, -1)
          : [page.title],
      });
      documentIds.push(documentId);
    }

    this.pageDocIds.set(page.id, documentIds);
  }

  // Remove all index entries belonging to a page
  discardPage (pageId: string): void {
    const ids = this.pageDocIds.get(pageId);

    if (!ids) return;
    for (const id of ids) {
      if (this.index.has(id)) this.index.discard(id);
    }
    this.pageDocIds.delete(pageId);
  }

  serialize (): string {
    return JSON.stringify(this.index);
  }
}

// Yields sections of rendered HTML split by heading tags
function * splitPageIntoSections (html: string): Generator<{
  anchor: string;
  titles: string[];
  text: string;
}> {
  // Split produces 3 groups per heading: level, inner HTML, content after
  const result = html.split(headingRegex);

  // Remove text before the first heading
  result.shift();
  const parentTitles: string[] = [];

  for (let index = 0; index < result.length; index += 3) {
    const level = Number.parseInt(result[index]) - 1;
    const heading = result[index + 1];
    const headingResult = headingContentRegex.exec(heading);
    const title = stripHtml(headingResult?.[1] ?? '').trim();
    const anchor = headingResult?.[2] ?? '';
    const content = result[index + 2];

    if (!title || !content) continue;

    // Build the title hierarchy up to the current level
    const titles = parentTitles.slice(0, level);

    titles[level] = title;
    yield {
      anchor,
      titles: titles.filter(Boolean),
      text: stripHtml(content),
    };

    // Track parent titles for nested heading breadcrumbs
    if (level === 0) {
      parentTitles.length = 0;
      parentTitles[0] = title;
    } else {
      parentTitles[level] = title;
    }
  }
}
