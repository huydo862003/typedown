export interface CodegenOptions {
  /** Content directory relative to the Vite root (for import.meta.glob) */
  contentDir: string;
  /** Import path for the theme's Layout component */
  layoutImport: string;
  /** Import path for the theme's NotFound component */
  notFoundImport: string;
  /** Package name for importing createTypedownApp */
  typerighterPackage: string;
}

// Generate the client hydration entry source
export function generateClientEntry (options: CodegenOptions): string {
  const glob = `/${options.contentDir}/**/*.td`;

  return `\
import { createTypedownApp } from '${options.typerighterPackage}/dist/client';
import Layout from '${options.layoutImport}';

const pages = import.meta.glob('${glob}');

async function loadPageModule(pagePath) {
  const key = '/${options.contentDir}/' + pagePath.replace(/^\\//, '') + '.td';
  const altKey = '/${options.contentDir}/' + pagePath.replace(/^\\//, '') + '/index.td';
  const loader = pages[key] || pages[altKey];
  if (!loader) return undefined;
  return loader();
}

async function main() {
  const { app } = await createTypedownApp(loadPageModule, Layout);
  app.mount('#app');
}

main();
`;
}

// Generate the SSR entry source
export function generateSsrEntry (options: CodegenOptions): string {
  const glob = `/${options.contentDir}/**/*.td`;

  return `\
import { createTypedownApp } from '${options.typerighterPackage}/dist/client';
import { renderToString } from 'vue/server-renderer';
import Layout from '${options.layoutImport}';

const pages = import.meta.glob('${glob}', { eager: true });

function loadPageModule(pagePath) {
  const key = '/${options.contentDir}/' + pagePath.replace(/^\\//, '') + '.td';
  const altKey = '/${options.contentDir}/' + pagePath.replace(/^\\//, '') + '/index.td';
  return Promise.resolve(pages[key] || pages[altKey]);
}

export async function render(url) {
  const { app, router } = await createTypedownApp(loadPageModule, Layout);

  await router.go(url, { replace: true });

  const html = await renderToString(app);

  return {
    html,
    pageData: router.route.data,
  };
}
`;
}
