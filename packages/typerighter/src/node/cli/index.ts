import pc from 'picocolors';
import {
  createLogger, createServer,
} from 'vite';
import {
  buildSite,
} from '../build';
import {
  typedown,
} from '../plugin/vite';

export async function cli () {
  const argv = process.argv.slice(2);
  const command = argv[0];
  const root = argv[1] ?? process.cwd();

  try {
    if (!command || command === 'dev') {
    // Skip configFile so user's vite.config.ts does not duplicate the typedown plugin
      const server = await createServer({
        root,
        configFile: false,
        plugins: [typedown()],
      });

      await server.listen();
      server.printUrls();
    } else if (command === 'build') {
      await buildSite({
        root,
      });
      // RPC WebSocket keeps the event loop alive, exit explicitly after build
      process.exit(0);
    } else if (command === 'init') {
      const {
        initialize,
      } = await import('./init');

      await initialize(root);
    } else if (command === 'preview') {
      const {
        preview,
      } = await import('vite');
      const server = await preview({
        root,
      });

      server.printUrls();
    } else {
      logErrorAndExit(`unknown command "${command}".`);
    }
  } catch (error) {
    logErrorAndExit(`${command ?? 'dev'} error:`, error);
  }
}

function logErrorAndExit (message: string, error?: unknown): never {
  const logger = createLogger();
  const parts = [pc.red(message)];

  if (error instanceof Error) {
    parts.push(error.message);
    if (error.stack) parts.push(error.stack);
  }

  logger.error(parts.join('\n'));
  process.exit(1);
}
