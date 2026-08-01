export interface HtmlTemplateContext {
  /** The server-rendered HTML content */
  content: string;
  /** Page title */
  title: string;
  /** Page description from frontmatter */
  description: string;
  /** Relative path to the client entry JS file */
  clientEntry: string;
  /** CSS file paths to preload */
  cssFiles: string[];
  /** JS file paths to preload (besides clientEntry) */
  jsFiles: string[];
  /** The base path (e.g. "/" or "/docs/") */
  base: string;
  /** HTML lang attribute (default: "en") */
  lang?: string;
}

// Generate the full HTML document for a pre-rendered page
export function renderHtml (context: HtmlTemplateContext): string {
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
    <title>${escapeHtmlAttr(context.title)}</title>
    <meta name="description" content="${escapeHtmlAttr(context.description)}">
${cssLinks}
${modulePreloads}
  </head>
  <body>
    <div id="app">${context.content}</div>
    <script type="module" src="${context.base}${context.clientEntry}"></script>
  </body>
</html>`;
}

// Escape characters for safe use in HTML attributes
function escapeHtmlAttr (string_: string): string {
  return string_
    .replaceAll('&', '&amp;')
    .replaceAll('"', '&quot;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;');
}
