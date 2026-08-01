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
  generateClientEntry, generateSsrEntry, type CodegenOptions,
} from './codegen';

export interface BuildOptions {
  /** Root directory of the Vite project */
  root: string;
  /** Output directory for the final build (default: "dist") */
  outDir?: string;
  /** Base public path (default: "/") */
  base?: string;
  /** Import path for the theme's Layout component */
  layoutImport: string;
  /** Import path for the theme's NotFound component */
  notFoundImport: string;
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

  // Lazy-import to avoid triggering top-level await at module load
  const { tdContext } = await import('../lib/typedown-context');

  const config = await tdContext.getConfig();
  const contentDir = config.contentDir;

  // Generate entry files in a temp directory
  const tempDir = await fs.mkdtemp(path.join(os.tmpdir(), 'typerighter-'));

  const codegenOptions: CodegenOptions = {
    contentDir,
    layoutImport: options.layoutImport,
    notFoundImport: options.notFoundImport,
    typerighterPackage: 'typerighter',
  };

  const clientEntryPath = path.join(tempDir, 'client-entry.js');
  const ssrEntryPath = path.join(tempDir, 'ssr-entry.js');

  await fs.writeFile(clientEntryPath, generateClientEntry(codegenOptions));
  await fs.writeFile(ssrEntryPath, generateSsrEntry(codegenOptions));

  logger.info('Phase 1: Building client bundle...');

  await build({
    ...options.viteConfig,
    root,
    base,
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
    build: {
      ...options.viteConfig?.build,
      outDir: ssrOutDir,
      ssr: ssrEntryPath,
      emptyOutDir: true,
    },
  });

  logger.info('Phase 3: Pre-rendering pages...');

  const files = await tdContext.listFiles();
  const pagePaths = files.map((file) => {
    const withoutExtension = file.replace(/\.td$/, '');

    return withoutExtension === 'index'
      ? '/'
      : `/${withoutExtension}`;
  });

  await renderPages({
    ssrEntryPath: path.join(ssrOutDir, 'ssr-entry.js'),
    clientOutDir,
    outDir,
    base,
    pagePaths,
  });

  // Copy client assets to the final output directory
  const clientAssetsDir = path.join(clientOutDir, 'assets');
  const outAssetsDir = path.join(outDir, 'assets');

  await fs.cp(clientAssetsDir, outAssetsDir, {
    recursive: true,
  }).catch(() => {
    // No assets to copy
  });

  // Clean up temp and intermediate directories
  await Promise.all([
    fs.rm(tempDir, {
      recursive: true,
      force: true,
    }),
    fs.rm(clientOutDir, {
      recursive: true,
      force: true,
    }),
    fs.rm(ssrOutDir, {
      recursive: true,
      force: true,
    }),
  ]);

  logger.info(`Build complete. Output: ${outDir}`);
}
