import path from 'node:path';
import fs from 'node:fs/promises';
import {
  build, type InlineConfig,
} from 'vite';
import {
  prerenderHtmlPages,
} from './prerender';
import {
  generateAppEntry, generateSsrEntry,
} from '../plugin/vite/codegen';
import {
  typedown,
} from '../plugin/vite';
import {
  ProgressLogger,
} from '../lib/progress';
import type {
  AppContext,
} from '../context';

const DEFAULT_LAYOUT_IMPORT = 'typerighter/client/theme-default';

export interface BuildOptions {
  /** Output directory for the final build (default: "dist") */
  outDir?: string;
  /** Base public path (default: "/") */
  base?: string;
  /** Additional Vite config overrides */
  viteConfig?: InlineConfig;
}

// Build the site into static HTML files
export async function buildSite (ctx: AppContext, options: BuildOptions = {}): Promise<void> {
  const { root, logger } = ctx;
  const outDir = path.resolve(root, options.outDir ?? 'dist');
  const base = options.base ?? '/';

  const clientOutDir = path.join(outDir, '.client');
  const ssrOutDir = path.join(outDir, '.server');

  // Clear stale output from previous builds
  await fs.rm(outDir, { recursive: true, force: true });

  // 1. Fetch project metadata from the RPC server
  const tdContext = await ctx.getTdContext();

  const [
    config,
    files,
    sidebarGroups,
  ] = await Promise.all([
    tdContext.getConfig(),
    tdContext.listFiles(),
    tdContext.listFilesGroupedBySchema(),
  ]);

  const hasIndex = files.some((file) => file === 'index.td');
  const siteConfig = JSON.stringify({ title: config.siteTitle, description: config.siteDescription });
  const siteData = JSON.stringify({ sidebarGroups });

  // 2. Generate entry files inside the project so Vite can resolve 'typerighter/*' imports
  const tempDir = path.join(root, 'node_modules', '.typerighter');
  await fs.mkdir(tempDir, { recursive: true });
  const clientEntryPath = path.join(tempDir, 'client-entry.js');
  const ssrEntryPath = path.join(tempDir, 'ssr-entry.js');

  await Promise.all([
    fs.writeFile(clientEntryPath, generateAppEntry({
      contentDir: config.contentDir,
      siteTitle: config.siteTitle,
      siteDescription: config.siteDescription,
      hasIndex,
      sidebarGroups,
    })),
    fs.writeFile(ssrEntryPath, generateSsrEntry({
      contentDir: config.contentDir,
      layoutImport: DEFAULT_LAYOUT_IMPORT,
      siteConfig,
      siteData,
    })),
  ]);

  try {
    const plugins = typedown({ context: ctx });
    const customLogger = ctx.createViteLogger();

    // 3. Build the client bundle (JS/CSS for the browser)
    const phase1 = new ProgressLogger(logger, 'Building client bundle...');

    await build({
      ...options.viteConfig,
      root,
      base,
      plugins,
      customLogger,
      logLevel: 'silent',
      build: {
        ...options.viteConfig?.build,
        outDir: clientOutDir,
        ssrManifest: true,
        emptyOutDir: true,
        rollupOptions: {
          ...options.viteConfig?.build?.rollupOptions,
          input: clientEntryPath,
        },
      },
    });

    const clientChunks = await fs.readdir(path.join(clientOutDir, 'assets')).catch(() => []);

    phase1.done(`Client bundle (${clientChunks.length} chunks)`);

    // 4. Build the SSR bundle (Node-runnable entry for pre-rendering)
    const phase2 = new ProgressLogger(logger, 'Building SSR bundle...');

    await build({
      ...options.viteConfig,
      root,
      base,
      plugins,
      customLogger,
      logLevel: 'silent',
      build: {
        ...options.viteConfig?.build,
        outDir: ssrOutDir,
        ssr: ssrEntryPath,
        emptyOutDir: true,
      },
    });

    phase2.done('SSR bundle');

    // 5. Pre-render each page to a static HTML file
    const pagePaths = files.map((file) => {
      const withoutExtension = file.replace(/\.td$/, '');

      return withoutExtension === 'index'
        ? '/'
        : `/${withoutExtension}`;
    });

    if (!hasIndex) {
      pagePaths.push('/');
    }

    const phase3 = new ProgressLogger(logger, 'Pre-rendering pages...');

    await prerenderHtmlPages({
      ssrEntryPath: path.join(ssrOutDir, 'ssr-entry.js'),
      clientOutDir,
      outDir,
      base,
      pagePaths,
      progress: phase3,
    });

    phase3.done(`Pre-rendered ${pagePaths.length} pages`);

    // 6. Copy assets to the final output directory
    const clientAssetsDir = path.join(clientOutDir, 'assets');
    const outAssetsDir = path.join(outDir, 'assets');

    await fs.cp(clientAssetsDir, outAssetsDir, {
      recursive: true,
    }).catch(() => {
      // No assets to copy
    });

    await copyContentAssets(path.join(root, config.contentDir), outDir);
  } finally {
    // 7. Clean up intermediate directories
    await Promise.all([
      fs.rm(tempDir, { recursive: true, force: true }),
      fs.rm(clientOutDir, { recursive: true, force: true }),
      fs.rm(ssrOutDir, { recursive: true, force: true }),
    ]);
  }

  logger.log(`\nBuild complete. Output: ${outDir}`);
}

// Copy non-.td files from content directory to output, preserving structure
async function copyContentAssets (contentDir: string, outDir: string): Promise<void> {
  const entries = await fs.readdir(contentDir, {
    recursive: true,
    withFileTypes: true,
  });

  const copies = entries
    .filter((entry) => entry.isFile() && !entry.name.endsWith('.td'))
    .map(async (entry) => {
      const src = path.join(entry.parentPath, entry.name);
      const relative = path.relative(contentDir, src);
      const dest = path.join(outDir, relative);

      return fs.mkdir(path.dirname(dest), { recursive: true })
        .then(() => fs.copyFile(src, dest));
    });

  await Promise.all(copies);
}
