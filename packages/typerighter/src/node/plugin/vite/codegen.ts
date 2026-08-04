import {
  VIRTUAL_INDEX_ID,
} from './constants';
import type {
  SidebarGroups,
} from '@/shared';

export interface AppEntryOptions {
  /** Content directory relative to project root */
  contentDir: string;
  /** Site title */
  siteTitle: string;
  /** Site description */
  siteDescription: string;
  /** Whether the vault has an index.td file */
  hasIndex: boolean;
  /** Content files grouped by schema for the sidebar */
  sidebarGroups: SidebarGroups;
}

export interface SsrEntryOptions {
  /** Content directory relative to project root */
  contentDir: string;
  /** Import path for the theme */
  layoutImport: string;
  /** Serialized siteConfig JSON */
  siteConfig: string;
  /** Serialized siteData JSON */
  siteData: string;
  /** Whether the vault has an index.td file */
  hasIndex: boolean;
}

// Generate the virtual app entry module
export function generateAppEntry (options: AppEntryOptions): string {
  const {
    contentDir, hasIndex,
  } = options;
  const glob = `/${contentDir}/**/*.td`;
  const indexPageEntry = hasIndex
    ? ''
    : `pages['/${contentDir}/${VIRTUAL_INDEX_ID}'] = () => import('${VIRTUAL_INDEX_ID}');`;
  const indexFallback = hasIndex
    ? ''
    : ` || (pagePath === '/' ? pages['/${contentDir}/${VIRTUAL_INDEX_ID}'] : undefined)`;

  const siteConfig = JSON.stringify({
    title: options.siteTitle,
    description: options.siteDescription,
  });

  const siteData = JSON.stringify({
    sidebarGroups: options.sidebarGroups,
  });

  return `
import 'typerighter/style.css';
import { createTypedownApp } from 'typerighter/client';
import theme from 'typerighter/client/theme-default';

const pages = import.meta.glob('${glob}');
${indexPageEntry}

async function loadPageModule(pagePath) {
  const key = '/${contentDir}/' + pagePath.replace(/^\\//, '') + '.td';
  const altKey = '/${contentDir}/' + pagePath.replace(/^\\//, '') + '/index.td';
  const loader = pages[key] || pages[altKey]${indexFallback};
  if (!loader) return undefined;
  return loader();
}

const { app } = await createTypedownApp(loadPageModule, theme.Layout, ${siteConfig}, ${siteData});
app.mount('#app');
`;
}

// Generate the virtual index page SFC
export function generateIndexSfc (groups: Record<string, unknown[]>): string {
  const data = JSON.stringify(groups).replace(/'/g, '\\\'');
  const pageData = JSON.stringify({
    schema: '',
    frontmatter: {},
    headings: [],
    title: 'Index',
  });

  return `<script>
import { TdOverview } from 'typerighter/client/theme-default';
export const __pageData = JSON.parse(${JSON.stringify(pageData)})
export default { name: "index", components: { TdOverview } }
</script>
<template><TdOverview :groups='${data}' /></template>`;
}

// Generate the SSR entry module used for pre-rendering
export function generateSsrEntry (options: SsrEntryOptions): string {
  const glob = `/${options.contentDir}/**/*.td`;
  const indexPageEntry = options.hasIndex
    ? ''
    : `pages['/${options.contentDir}/${VIRTUAL_INDEX_ID}'] = import('${VIRTUAL_INDEX_ID}');`;
  const indexFallback = options.hasIndex
    ? ''
    : ` || (pagePath === '/' ? pages['/${options.contentDir}/${VIRTUAL_INDEX_ID}'] : undefined)`;

  return `
import { createTypedownApp } from 'typerighter/client';
import { renderToString } from 'vue/server-renderer';
import theme from '${options.layoutImport}';

const pages = import.meta.glob('${glob}', { eager: true });
${indexPageEntry}

function loadPageModule(pagePath) {
  const key = '/${options.contentDir}/' + pagePath.replace(/^\\//, '') + '.td';
  const altKey = '/${options.contentDir}/' + pagePath.replace(/^\\//, '') + '/index.td';
  return Promise.resolve(pages[key] || pages[altKey]${indexFallback});
}

export async function render(url) {
  const { app, router } = await createTypedownApp(loadPageModule, theme.Layout, ${options.siteConfig}, ${options.siteData});
  await router.go(url, { replace: true });
  const html = await renderToString(app);
  return { html, pageData: router.route.data };
}
`;
}
