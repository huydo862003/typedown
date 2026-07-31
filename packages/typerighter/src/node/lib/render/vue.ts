import type {
  TdBuiltResource,
} from '@typerighter/rpc-client';
import type {
  MarkdownRenderer,
} from '../markdown';
import type {
  MarkdownEnv,
  PageData,
} from '@/shared';
import {
  getResourceTitle,
} from '@/shared';

export interface VueRenderResult {
  /** The rendered Vue SFC string */
  vueSrc: string;
  /** Page metadata */
  pageData: PageData;
}

// Render a built resource to a Vue SFC string
export async function renderToVueSfc (
  md: MarkdownRenderer,
  resource: TdBuiltResource,
  filePath: string,
): Promise<VueRenderResult> {
  const env: MarkdownEnv = {
    path: filePath,
    relativePath: filePath,
    cleanUrls: true,
  };

  const html = await md.renderAsync(resource.content, env);

  const title = env.title || getResourceTitle(resource.header, filePath);

  const pageData = {
    schema: resource.schema,
    frontmatter: resource.header,
    headings: env.headers ?? [],
    title,
  };

  const pageDataJson = JSON.stringify(JSON.stringify(pageData));

  const htmlJson = JSON.stringify(html);

  const vueSrc = [
    '<script>',
    `export const __pageData = JSON.parse(${pageDataJson})`,
    `export default { name: ${JSON.stringify(filePath)}, data() { return { __html: ${htmlJson} } } }`,
    '</script>',
    '<template><div v-html="__html" /></template>',
  ].join('\n');

  return {
    vueSrc,
    pageData,
  };
}
