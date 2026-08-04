import fs from 'node:fs/promises';
import path from 'node:path';
import {
  escapeHtml,
} from '@/shared';
import type {
  ProgressLogger,
} from '../lib/progress';

export interface PrerenderContext {
  /** Absolute path to the SSR bundle entry */
  ssrEntryPath: string;
  /** Absolute path to the client output directory */
  clientOutDir: string;
  /** Absolute path to the final output directory */
  outDir: string;
  /** The base path (e.g. "/") */
  base: string;
  /** List of page paths to render (e.g. ["/", "/posts/hello"]) */
  pagePaths: string[];
  /** Progress logger for reporting render progress */
  progress?: ProgressLogger;
}


// Pre-render all pages to static HTML files
export async function prerenderHtmlPages (context: PrerenderContext): Promise<void> {
  // 1. Load the SSR bundle and resolve client assets (JS/CSS paths)
  const ssrModule = await import(context.ssrEntryPath);
  const { clientEntry, cssFiles, jsFiles } = await resolveClientAssets(context.clientOutDir);

  // 2. Render all pages concurrently and write to disk
  const totalPages = context.pagePaths.length;
  let renderedPages = 0;

  await Promise.all(context.pagePaths.map(async (pagePath) => {
    const result = await ssrModule.render(pagePath);

    const html = generateHtmlDocument({
      content: result.html,
      title: result.pageData.title,
      description: result.pageData.frontmatter.description !== undefined
        ? String(result.pageData.frontmatter.description)
        : '',
      clientEntry,
      cssFiles,
      jsFiles,
      base: context.base,
    });

    const fileName = pagePath === '/'
      ? 'index.html'
      : `${pagePath.replace(/^\//, '')}.html`;
    const filePath = path.join(context.outDir, fileName);

    await fs.mkdir(path.dirname(filePath), { recursive: true });
    await fs.writeFile(filePath, html);

    renderedPages++;
    context.progress?.update(renderedPages, totalPages);
  }));
}

// Generate the full HTML document shell for a pre-rendered page
interface HtmlDocumentContext {
  content: string;
  title: string;
  description: string;
  clientEntry: string;
  cssFiles: string[];
  jsFiles: string[];
  base: string;
  lang?: string;
}

function generateHtmlDocument (context: HtmlDocumentContext): string {
  const cssLinks = context.cssFiles
    .map((file) => `    <link rel="stylesheet" href="${context.base}${file}">`)
    .join('\n');

  const modulePreloads = context.jsFiles
    .map((file) => `    <link rel="modulepreload" href="${context.base}${file}">`)
    .join('\n');

  return `<!DOCTYPE html>
<html lang="${context.lang ?? 'en'}">
  <head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>${escapeHtml(context.title)}</title>
    <meta name="description" content="${escapeHtml(context.description)}">
${cssLinks}
${modulePreloads}
  </head>
  <body>
    <div id="app">${context.content}</div>
    <script type="module" src="${context.base}${context.clientEntry}"></script>
  </body>
</html>`;
}

// Read the Vite manifest to find the client entry and asset files
async function resolveClientAssets (clientOutDir: string): Promise<{
  clientEntry: string;
  cssFiles: string[];
  jsFiles: string[];
}> {
  const manifestPath = path.join(clientOutDir, '.vite', 'manifest.json');
  const raw = await fs.readFile(manifestPath, 'utf-8');
  const manifest = JSON.parse(raw) as Record<string, {
    file: string;
    isEntry?: boolean;
    css?: string[];
    imports?: string[];
  }>;

  let clientEntry = '';
  const cssFiles: string[] = [];
  const jsFiles: string[] = [];

  for (const chunk of Object.values(manifest)) {
    if (chunk.isEntry) {
      clientEntry = chunk.file;

      if (chunk.css) {
        cssFiles.push(...chunk.css);
      }
    } else if (chunk.file.endsWith('.js')) {
      jsFiles.push(chunk.file);
    }
  }

  return { clientEntry, cssFiles, jsFiles };
}
