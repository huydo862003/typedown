import fs from 'node:fs/promises';
import path from 'node:path';
import type {
  PageData,
} from '@/shared';
import {
  renderHtml, type HtmlTemplateContext,
} from './html';

export interface RenderContext {
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
}

interface SsrModule {
  render: (url: string) => Promise<{
    html: string;
    pageData: PageData;
  }>;
}

// Pre-render all pages to static HTML files
export async function renderPages (context: RenderContext): Promise<void> {
  const ssrModule = await import(context.ssrEntryPath) as SsrModule;
  const { clientEntry, cssFiles, jsFiles } = await resolveClientAssets(context.clientOutDir);

  // Render pages concurrently in batches
  const BATCH_SIZE = 8;

  for (let i = 0; i < context.pagePaths.length; i += BATCH_SIZE) {
    const batch = context.pagePaths.slice(i, i + BATCH_SIZE);

    await Promise.all(batch.map(async (pagePath) => {
      const result = await ssrModule.render(pagePath);

      const htmlContext: HtmlTemplateContext = {
        content: result.html,
        title: result.pageData.title,
        description: result.pageData.frontmatter.description !== undefined
          ? String(result.pageData.frontmatter.description)
          : '',
        clientEntry,
        cssFiles,
        jsFiles,
        base: context.base,
      };

      const html = renderHtml(htmlContext);

      // Write /foo -> /foo.html, / -> /index.html
      const fileName = pagePath === '/'
        ? 'index.html'
        : `${pagePath.replace(/^\//, '')}.html`;
      const filePath = path.join(context.outDir, fileName);

      await fs.mkdir(path.dirname(filePath), {
        recursive: true,
      });
      await fs.writeFile(filePath, html);
    }));
  }
}

// Scan the client output directory for the entry JS and CSS files
async function resolveClientAssets (clientOutDir: string): Promise<{
  clientEntry: string;
  cssFiles: string[];
  jsFiles: string[];
}> {
  const assetsDir = path.join(clientOutDir, 'assets');
  const empty: string[] = [];
  const files = await fs.readdir(assetsDir).catch(() => empty);

  const cssFiles: string[] = [];
  const jsFiles: string[] = [];
  let clientEntry = '';

  for (const file of files) {
    if (file.endsWith('.css')) {
      cssFiles.push(`assets/${file}`);
    } else if (file.endsWith('.js')) {
      if (file.startsWith('client-entry') || file.startsWith('index')) {
        clientEntry = `assets/${file}`;
      } else {
        jsFiles.push(`assets/${file}`);
      }
    }
  }

  if (!clientEntry && jsFiles.length > 0) {
    clientEntry = jsFiles[0];
    jsFiles.splice(0, 1);
  }

  return {
    clientEntry,
    cssFiles,
    jsFiles,
  };
}
