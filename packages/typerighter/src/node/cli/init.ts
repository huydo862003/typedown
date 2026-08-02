import fs from 'node:fs/promises';
import path from 'node:path';
import {
  cancel, intro, isCancel, log, outro, text,
} from '@clack/prompts';
import {
  escapeHtml,
} from '../build/html';

// Interactive project scaffolding
export async function initialize (targetDirectory: string): Promise<void> {
  intro('typedown');

  const defaultName = path.basename(path.resolve(targetDirectory));

  const projectName = await text({
    message: 'Project name',
    placeholder: defaultName,
    defaultValue: defaultName,
  });

  if (isCancel(projectName)) {
    cancel('Cancelled.');
    process.exit(0);
  }

  const siteTitle = await text({
    message: 'Site title',
    placeholder: projectName,
    defaultValue: projectName,
  });

  if (isCancel(siteTitle)) {
    cancel('Cancelled.');
    process.exit(0);
  }

  const siteDescription = await text({
    message: 'Site description',
    placeholder: 'A typedown site',
    defaultValue: 'A typedown site',
  });

  if (isCancel(siteDescription)) {
    cancel('Cancelled.');
    process.exit(0);
  }

  await scaffold(targetDirectory, {
    projectName,
    siteTitle,
    siteDescription,
  });

  const steps = [];

  if (targetDirectory !== '.') {
    steps.push(`cd ${targetDirectory}`);
  }

  steps.push('pnpm install', 'pnpm dev');
  log.info(steps.join('\n'));

  outro('Done');
}

interface InitializeOptions {
  projectName: string;
  siteTitle: string;
  siteDescription: string;
}

function indexHtml (options: InitializeOptions): string {
  return `<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>${escapeHtml(options.siteTitle)}</title>
  </head>
  <body>
    <div id="app"></div>
  </body>
</html>
`;
}

function packageJson (options: InitializeOptions): string {
  return JSON.stringify({
    name: options.projectName,
    version: '0.0.0',
    private: true,
    type: 'module',
    scripts: {
      dev: 'typedown dev',
      build: 'typedown build',
      preview: 'typedown preview',
    },
    dependencies: {
      typerighter: '^0.1.3',
      vue: '^3.5.0',
    },
    devDependencies: {
      vite: '^8.0.0',
    },
  }, undefined, 2) + '\n';
}

function sampleContent (): string {
  return `---
_type: Article
title: "Hello, world"
---

Welcome to your new typedown site.
`;
}

function sampleSchema (): string {
  return `---
_type: schema
properties:
  title:
    type: string
  tags:
    type: list[string]
    optional: true
---
`;
}

async function scaffold (targetDirectory: string, options: InitializeOptions): Promise<void> {
  const root = path.resolve(targetDirectory);
  const contentDirectory = path.join(root, 'vault', 'content');
  const schemaDirectory = path.join(root, 'vault', 'schemas');

  await fs.mkdir(contentDirectory, {
    recursive: true,
  });
  await fs.mkdir(schemaDirectory, {
    recursive: true,
  });

  await Promise.all([
    writeFile(root, 'package.json', packageJson(options)),
    writeFile(root, 'vite.config.ts', viteConfig()),
    writeFile(root, 'typedown.yaml', typedownYaml(options)),
    writeFile(root, 'index.html', indexHtml(options)),
    writeFile(schemaDirectory, 'Article.td', sampleSchema()),
    writeFile(contentDirectory, 'hello.td', sampleContent()),
  ]);
}

function typedownYaml (options: InitializeOptions): string {
  return `version: "1.0.0"
vault:
  content_dir: vault/content
  schema_dir: vault/schemas
site:
  title: "${options.siteTitle}"
  description: "${options.siteDescription}"
`;
}

function viteConfig (): string {
  return `import { defineConfig } from "vite";
import { typedown } from "typerighter/vite";

export default defineConfig({
  plugins: [typedown()],
});
`;
}

async function writeFile (directory: string, name: string, content: string): Promise<void> {
  const filePath = path.join(directory, name);
  const exists = await fs.access(filePath).then(() => true)
    .catch(() => false);

  if (exists) {
    log.warn(`skip ${name} (already exists)`);

    return;
  }

  await fs.writeFile(filePath, content);
  log.success(`created ${name}`);
}
