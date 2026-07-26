import { EventEmitter } from 'node:events';

export interface RpcServerOptions {
  /** Project root (default: cwd) */
  root?: string;
  /** Bind address (default: "127.0.0.1") */
  addr?: string;
  /** Bind port (default: 4747) */
  port?: number;
}

export class RpcServer extends EventEmitter {
  constructor(options?: RpcServerOptions);

  /** The ws:// address the server is listening on, or null if not started */
  get address(): string | null;

  /** Whether the server is currently listening */
  get listening(): boolean;

  /** Start the server. Emits "listening" when ready */
  listen(callback?: () => void): this;

  /** Stop the server. Emits "close" when the process exits */
  close(): this;

  on(event: 'listening', listener: () => void): this;
  on(event: 'close', listener: (code: number | null, signal: string | null) => void): this;
  on(event: 'error', listener: (error: Error) => void): this;
}
