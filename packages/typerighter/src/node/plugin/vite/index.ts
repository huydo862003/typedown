import tailwindcss from '@tailwindcss/vite';
import vue from '@vitejs/plugin-vue';
import type {
  Plugin, ViteDevServer,
} from 'vite';
import {
  tdContext,
} from '../../lib/typedown-context';
import {
  renderToVueSfc,
} from '../../lib/render';
import {
  escapeHtml,
} from '../../build/html';
import {
  VIRTUAL_APP_ID, RESOLVED_VIRTUAL_APP_ID, VIRTUAL_INDEX_ID,
} from './constants';
import {
  generateAppEntry, generateIndexSfc,
} from './codegen';
import {
  BRAND_FAVICON_URI,
} from '@/shared/brand';

// Create the typedown vite plugin with the Vue plugin bundled
export function typedown (): Plugin[] {
  let server: ViteDevServer | undefined;

  const vuePlugin = vue({
    include: /\.(?:vue|td)$/,
  });

  const typedownPlugin: Plugin = {
    name: 'vite-plugin-typedown',

    enforce: 'pre',

    // Inject favicon, app entry script, and site title into index.html
    async transformIndexHtml (html) {
      const config = await tdContext.getConfig();

      return html
        .replace(
          /<title>.*?<\/title>/,
          `<title>${escapeHtml(config.siteTitle)}</title>`,
        )
        .replace(
          '</head>',
          `  <link rel="icon" type="image/svg+xml" href="${BRAND_FAVICON_URI}">\n</head>`,
        )
        .replace(
          '</body>',
          `  <script type="module" src="/${VIRTUAL_APP_ID}"></script>\n</body>`,
        );
    },

    // Resolve virtual modules
    resolveId (id) {
      if (id === '/' + VIRTUAL_APP_ID || id === VIRTUAL_APP_ID) {
        return RESOLVED_VIRTUAL_APP_ID;
      }
      if (id.endsWith(VIRTUAL_INDEX_ID)) {
        return id;
      }
    },

    // Serve virtual modules
    async load (id) {
      if (id === RESOLVED_VIRTUAL_APP_ID) {
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

        return generateAppEntry({
          ...config,
          hasIndex,
          sidebarGroups,
        });
      }

      if (id.endsWith(VIRTUAL_INDEX_ID)) {
        const groups = await tdContext.listFilesGroupedBySchema();

        return generateIndexSfc(groups);
      }
    },

    configureServer (devServer) {
      server = devServer;

      // Config changes affect the rendering pipeline itself, requires full reload
      tdContext.rpc.onConfigChanged(() => server && hmrFullReload(server));

      tdContext.rpc.onContentChanged(({
        content,
      }: {
        content: string;
      }) => {
        if (!server) return;

        for (const module_ of server.moduleGraph.idToModuleMap.values()) {
          if (module_.file?.endsWith(content)) {
            server.moduleGraph.invalidateModule(module_);
            server.hot.send({
              type: 'update',
              updates: [
                {
                  type: 'js-update' as const,
                  path: module_.url,
                  acceptedPath: module_.url,
                  timestamp: Date.now(),
                },
              ],
            });
            break;
          }
        }
      });
      tdContext.rpc.onContentCreated(() => server && hmrInvalidateAll(server));
      tdContext.rpc.onContentDeleted(() => server && hmrInvalidateAll(server));

      // Schema changes affect all pages using that schema
      tdContext.rpc.onSchemaChanged(() => server && hmrInvalidateAll(server));
      tdContext.rpc.onSchemaCreated(() => server && hmrInvalidateAll(server));
      tdContext.rpc.onSchemaDeleted(() => server && hmrInvalidateAll(server));

      // Serve a default index.html for SPA routing
      return () => {
        devServer.middlewares.use((request, result, next) => {
          if (!request.url || result.writableEnded) return next();

          if (request.url.startsWith('/@') || request.url.startsWith('/node_modules') || request.url.includes('.')) {
            return next();
          }

          tdContext.getConfig().then((config) => {
            const html = `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>${escapeHtml(config.siteTitle)}</title>
  <link rel="icon" type="image/svg+xml" href="${BRAND_FAVICON_URI}">
</head>
<body>
  <div id="app"></div>
  <script type="module" src="/${VIRTUAL_APP_ID}"></script>
</body>
</html>`;

            result.setHeader('Content-Type', 'text/html');
            result.end(html);
          })
            .catch(next);
        });
      };
    },

    async transform (_, id) {
      const cleanId = id.split('?')[0];

      if (!cleanId.endsWith('.td')) return;
      if (cleanId.includes(VIRTUAL_INDEX_ID)) return;

      const config = await tdContext.getConfig();
      const contentDirectory = config.contentDir;
      const relativePath = cleanId.includes(contentDirectory)
        ? cleanId.slice(cleanId.indexOf(contentDirectory) + contentDirectory.length + 1)
        : cleanId;

      const resource = await tdContext.getFile(relativePath);
      const {
        vueSrc,
      } = await renderToVueSfc(tdContext.md, resource, relativePath);

      return {
        code: vueSrc,
        map: null,
      };
    },

    handleHotUpdate ({
      file,
    }) {
      if (file.endsWith('.td')) return [];
    },
  };

  return [
    typedownPlugin,
    vuePlugin,
    ...tailwindcss(),
  ];
}

// Get all .td modules from the module graph
function getTdModules (server: ViteDevServer) {
  return [...server.moduleGraph.idToModuleMap.entries()]
    .filter(([id]) => id.endsWith('.td'))
    .map(([
      , module_,
    ]) => module_);
}

// Full page reload for config changes
function hmrFullReload (server: ViteDevServer): void {
  for (const module_ of getTdModules(server)) {
    server.moduleGraph.invalidateModule(module_);
  }

  server.hot.send({
    type: 'full-reload',
  });
}

// Invalidate all .td modules and trigger HMR
function hmrInvalidateAll (server: ViteDevServer): void {
  const modules = getTdModules(server);
  const updates = modules.map((module_) => {
    server.moduleGraph.invalidateModule(module_);

    return makeHmrUpdate(module_);
  });

  if (0 < updates.length) {
    server.hot.send({
      type: 'update',
      updates,
    });
  }
}

// Build an HMR update payload for a module
function makeHmrUpdate (module_: {
  url: string;
}) {
  return {
    type: 'js-update' as const,
    path: module_.url,
    acceptedPath: module_.url,
    timestamp: Date.now(),
  };
}
