import type {
  ResolvedConfig,
  ServerOptions,
} from 'vite';
import {
  createServer as _createServer,
} from 'vite';

export type ViteServerConfig = ResolvedConfig;
export type ViteServerOptions = ServerOptions;

// The rpc server knows all configs already
// This one is just a wrapper
// It's meant to spin up dev server if typerighter is spun up from the CLI (separate from vite, i guess)
export async function createViteServer (
  options: ViteServerOptions = {},
  config: ViteServerConfig,
) {
  return _createServer({
    root: config.root,
    base: config.base,
    cacheDir: config.cacheDir,
    plugins: [],
    server: options,
    customLogger: config.logger,
    configFile: config.configFile,
  });
}
