import path from 'node:path';
import fs from 'node:fs/promises';
import os from 'node:os';
import {
  build, type InlineConfig,
} from 'vite';
import {
  renderPages,
} from './render';
import {
  logger,
} from '../lib/logger';
import {
  generateAppEntry, generateSsrEntry,
} from '../plugin/vite/codegen';
import {
  typedown,
} from '../plugin/vite';
import { getTdContext } from '../lib';

const DEFAULT_LAYOUT_IMPORT = 'typerighter/client/theme-default';

export interface BuildOptions {
  /** Root directory of the Vite project */
  root: string;
  /** Output directory for the final build (default: "dist") */
  outDir?: string;
  /** Base public path (default: "/") */
  base?: string;
  /** Additional Vite config overrides */
  viteConfig?: InlineConfig;
}

// Build the site into static HTML files
export async function buildSite (options: BuildOptions): Promise<void> {
  const root = path.resolve(options.root);
  const outDir = path.resolve(root, options.outDir ?? 'dist');
  const base = options.base ?? '/';

  const clientOutDir = path.join(outDir, '.client');
  const ssrOutDir = path.join(outDir, '.server');

  const tdContext = await getTdContext();

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

  // Generate entry files inside the project so Vite can resolve 'typerighter/*' imports
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
    const plugins = typedown();

    logger.info('Phase 1: Building client bundle...');

    await build({
      ...options.viteConfig,
      root,
      base,
      plugins,
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

    logger.info('Phase 2: Building SSR bundle...');

    await build({
      ...options.viteConfig,
      root,
      base,
      plugins,
      build: {
        ...options.viteConfig?.build,
        outDir: ssrOutDir,
        ssr: ssrEntryPath,
        emptyOutDir: true,
      },
    });

    logger.info('Phase 3: Pre-rendering pages...');

    const pagePaths = files.map((file) => {
      const withoutExtension = file.replace(/\.td$/, '');

      return withoutExtension === 'index'
        ? '/'
        : `/${withoutExtension}`;
    });

    if (!hasIndex) {
      pagePaths.push('/');
    }

    await renderPages({
      ssrEntryPath: path.join(ssrOutDir, 'ssr-entry.js'),
      clientOutDir,
      outDir,
      base,
      pagePaths,
    });

    // Copy client assets (JS/CSS) to the final output directory
    const clientAssetsDir = path.join(clientOutDir, 'assets');
    const outAssetsDir = path.join(outDir, 'assets');

    await fs.cp(clientAssetsDir, outAssetsDir, {
      recursive: true,
    }).catch(() => {
      // No assets to copy
    });

    // Copy content assets (images, PDFs, etc.) to the output directory
    await copyContentAssets(path.join(root, config.contentDir), outDir);
  } finally {
    // Clean up temp and intermediate directories
    await Promise.all([
      fs.rm(tempDir, { recursive: true, force: true }),
      fs.rm(clientOutDir, { recursive: true, force: true }),
      fs.rm(ssrOutDir, { recursive: true, force: true }),
    ]);
  }

  logger.info(`Build complete. Output: ${outDir}`);
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
