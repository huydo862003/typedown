import fs from 'node:fs/promises';
import path from 'node:path';
import {
  cancel, intro, isCancel, log, outro, text,
} from '@clack/prompts';
import {
  escapeHtml,
} from '../build/html';
import {
  VIRTUAL_APP_ID,
} from '../plugin/vite/constants';
import {
  BRAND_FAVICON_URI,
} from '@/shared/brand';

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

  const steps = ['Done scaffolding.'];

  if (targetDirectory !== '.') {
    steps.push(`cd ${targetDirectory}`);
  }

  steps.push('pnpm install', 'pnpm dev');
  outro(steps.join('\n'));
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
    <link
      rel="icon"
      type="image/svg+xml"
      href="${BRAND_FAVICON_URI}"
    >
  </head>
  <body>
    <div id="app"></div>
    <script
      type="module"
      src="/${VIRTUAL_APP_ID}"
    ></script>
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
      dev: 'typerighter dev',
      build: 'typerighter build',
      preview: 'typerighter preview',
    },
    devDependencies: {
      typerighter: '^0.1.6',
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
    writeFile(root, 'typedown.yaml', typedownYaml(options)),
    writeFile(root, 'index.html', indexHtml(options)),
    writeFile(schemaDirectory, 'Article.td', sampleSchema(), {
      shouldLog: false,
    }),
    writeFile(contentDirectory, 'hello.td', sampleContent(), {
      shouldLog: false,
    }),
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

async function writeFile (
  directory: string,
  name: string,
  content: string,
  {
    shouldLog = true,
  }: {
    shouldLog?: boolean;
  } = {},
): Promise<void> {
  const filePath = path.join(directory, name);
  const exists = await fs.access(filePath).then(() => true)
    .catch(() => false);

  if (exists && shouldLog) {
    log.warn(`skip ${name} (already exists)`);

    return;
  }

  await fs.writeFile(filePath, content);
  if (shouldLog) log.success(`created ${name}`);
}
