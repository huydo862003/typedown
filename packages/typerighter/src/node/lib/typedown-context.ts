import path from 'node:path';
import {
  RpcServer,
} from '@typerighter/rpc-server';
import {
  RpcClient,
} from '@typerighter/rpc-client';
import type {
  TdBuiltResource, TdSiteConfig, TdSchemaInfo,
} from '@typerighter/rpc-client';
import {
  createMarkdownRenderer, type MarkdownRenderer,
} from './markdown';
import {
  logger,
} from './logger';
import type {
  SidebarGroups,
} from '@/shared';

// The context is always rooted at the current directory
// This is fine: `typedown.yaml` should be at project root + user can run within any dir nested in the project root
export class TypedownContext {
  private client: RpcClient;
  private _md: MarkdownRenderer;
  constructor (client: RpcClient, md: MarkdownRenderer) {
    this.client = client;
    this._md = md;
    this.registerNotificationHandlers(client);
  }

  private cachedConfig: TdSiteConfig | undefined;
  private cachedFiles: string[] | undefined;
  private cachedFilesGroupedBySchema: SidebarGroups | undefined;
  private cachedSchemas: string[] | undefined;
  private cachedSchemaMap = new Map<string, TdSchemaInfo>();
  private cachedFileMap = new Map<string, TdBuiltResource>();

  private configVersion = 0;

  private registerNotificationHandlers (client: RpcClient) {
    client.onConfigChanged((config: TdSiteConfig) => {
      this.cachedConfig = config;
      // Recreate the markdown renderer with the new config (e.g. basePath may have changed)
      const version = ++this.configVersion;

      createMarkdownRenderer(config).then((newMd) => {
        if (this.configVersion === version) {
          this._md = newMd;
        }
      })
        .catch((error) => {
          logger.error(`Failed to recreate markdown renderer: ${error}`);
        });
    });

    client.onContentChanged(({
      content,
    }: {
      content: string;
    }) => {
      this.cachedFileMap.delete(content);
      this.cachedFilesGroupedBySchema = undefined;
    });

    client.onContentCreated(() => {
      this.cachedFiles = undefined;
      this.cachedFilesGroupedBySchema = undefined;
    });

    client.onContentDeleted(({
      content,
    }: {
      content: string;
    }) => {
      this.cachedFiles = undefined;
      this.cachedFilesGroupedBySchema = undefined;
      this.cachedFileMap.delete(content);
    });

    client.onSchemaChanged(({
      schema,
    }: {
      schema: string;
    }) => {
      this.cachedSchemaMap.delete(schema);
    });

    client.onSchemaCreated(() => {
      this.cachedSchemas = undefined;
    });

    client.onSchemaDeleted(({
      schema,
    }: {
      schema: string;
    }) => {
      this.cachedSchemas = undefined;
      this.cachedSchemaMap.delete(schema);
    });
  }

  get rpc (): RpcClient {
    return this.client;
  }

  /* File operations */

  async getFile (filePath: string): Promise<TdBuiltResource> {
    return this.rpc.requestFile(filePath);
  }

  async getFiles (paths: string[]): Promise<TdBuiltResource[]> {
    const results = await this.rpc.requestFiles(paths);

    for (const [
      index,
      filePath,
    ] of paths.entries()) {
      this.cachedFileMap.set(filePath, results[index]);
    }

    return results;
  }

  async listFiles (): Promise<string[]> {
    if (this.cachedFiles) return this.cachedFiles;

    this.cachedFiles = await this.rpc.listVault();

    return this.cachedFiles;
  }

  async listFilesGroupedBySchema (): Promise<SidebarGroups> {
    if (this.cachedFilesGroupedBySchema) return this.cachedFilesGroupedBySchema;

    const raw = await this.rpc.listFilesGroupedBySchema();
    // serde_wasm_bindgen converts HashMap to a JS Map, convert to plain object
    const result: SidebarGroups = raw instanceof Map
      ? Object.fromEntries(raw)
      : raw ?? {};

    this.cachedFilesGroupedBySchema = result;

    return result;
  }

  /* Project operations */

  async getConfig (): Promise<TdSiteConfig> {
    if (this.cachedConfig) return this.cachedConfig;

    this.cachedConfig = await this.rpc.getConfig();

    return this.cachedConfig;
  }

  // Get the asset directory for a given file
  async getAssetDir (filePath: string): Promise<string> {
    const config = await this.getConfig();

    return path.join(path.dirname(filePath), config.assetsDir.path);
  }

  async getSchema (schema: string): Promise<TdSchemaInfo> {
    const cached = this.cachedSchemaMap.get(schema);

    if (cached) return cached;

    const result = await this.rpc.getSchema(schema);

    this.cachedSchemaMap.set(schema, result);

    return result;
  }

  async listSchemas (): Promise<string[]> {
    if (this.cachedSchemas) return this.cachedSchemas;

    this.cachedSchemas = await this.rpc.listSchemas();

    return this.cachedSchemas;
  }

  get md (): MarkdownRenderer {
    return this._md;
  }
}

// Start the RPC server, connect the client, and create the markdown renderer
async function initializeTypedownContext (): Promise<TypedownContext> {
  const server = new RpcServer();

  await new Promise<void>((resolve, reject) => {
    server.once('error', reject);
    server.listen(() => {
      server.removeListener('error', reject);
      resolve();
    });
  });

  const address = server.address;
  const port = server.port;

  if (address === undefined || port === undefined) {
    throw new Error('RPC server started but address/port not available');
  }

  const client = await RpcClient.connect(new URL(address).hostname, port);
  const config = await client.getConfig();
  const md = await createMarkdownRenderer(config);
  const context = new TypedownContext(client, md);

  // Unref the child so it does not prevent Node from exiting after vite build finishes
  server.unref();

  function cleanup () {
    server.close();
  }

  process.on('exit', cleanup);
  process.on('SIGINT', () => {
    cleanup();
    process.exit(0);
  });
  process.on('SIGTERM', () => {
    cleanup();
    process.exit(0);
  });

  return context;
}

let _tdContext: TypedownContext | undefined;

export async function getTdContext (): Promise<TypedownContext> {
  if (!_tdContext) {
    _tdContext = await initializeTypedownContext();
  }

  return _tdContext;
}
